//! Line-oriented recursive-descent parser that converts `.bub` source into a [`Vec<Node>`].

pub(super) mod helpers;
pub(super) mod stmt;

use indexmap::IndexMap;

use crate::compiler::ast::{LineVariant, Node, OptionItem, Stmt};
use crate::error::{DialogueError, Result};

use helpers::{leading_spaces, parse_line_stmt, split_trailing_tags};
use stmt::parse_option_text;

// ── public entry point ────────────────────────────────────────────────────────

/// Parses a single `.bub` source string into a list of [`Node`]s.
pub fn parse(file: &str, source: &str) -> Result<Vec<Node>> {
    let mut p = Parser::new(file, source);
    p.parse_file()
}

// ── parser state ──────────────────────────────────────────────────────────────

pub(super) struct Parser<'src> {
    pub(super) file: &'src str,
    pub(super) lines: Vec<(usize, &'src str)>,
    pub(super) pos: usize,
    pub(super) id_counter: usize,
}

impl<'src> Parser<'src> {
    fn new(file: &'src str, source: &'src str) -> Self {
        let lines = source
            .lines()
            .enumerate()
            .map(|(i, l)| (i + 1, l))
            .collect();
        Self {
            file,
            lines,
            pos: 0,
            id_counter: 0,
        }
    }

    pub(super) fn next_id(&mut self) -> String {
        self.id_counter += 1;
        format!("blk{}", self.id_counter)
    }

    pub(super) fn peek(&self) -> Option<(usize, &'src str)> {
        self.lines.get(self.pos).copied()
    }

    pub(super) fn advance(&mut self) -> Option<(usize, &'src str)> {
        let line = self.lines.get(self.pos).copied();
        self.pos += 1;
        line
    }

    pub(super) fn err(&self, line: usize, msg: impl Into<String>) -> DialogueError {
        DialogueError::Parse {
            file: self.file.to_owned(),
            line,
            message: msg.into(),
        }
    }

    pub(super) fn skip_blank_and_comments(&mut self) {
        while let Some((_, content)) = self.peek() {
            let t = content.trim();
            if t.is_empty() || t.starts_with("//") {
                self.advance();
            } else {
                break;
            }
        }
    }
}

// ── file / node parsing ───────────────────────────────────────────────────────

impl Parser<'_> {
    fn parse_file(&mut self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        loop {
            self.skip_blank_and_comments();
            if self.peek().is_none() {
                break;
            }
            nodes.push(self.parse_node()?);
        }
        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<Node> {
        let headers = self.parse_headers()?;
        let title = headers
            .get("title")
            .cloned()
            .ok_or_else(|| DialogueError::Parse {
                file: self.file.to_owned(),
                line: self.pos,
                message: "node is missing a `title:` header".into(),
            })?;
        let tags = headers
            .get("tags")
            .map(|s| s.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default();
        let when_src = headers.get("when").cloned();
        let mut extra = headers;
        extra.shift_remove("title");
        extra.shift_remove("tags");
        extra.shift_remove("when");

        self.expect_body_start()?;
        let body = self.parse_body(0)?;
        self.expect_node_end()?;

        Ok(Node {
            title,
            tags,
            headers: extra,
            when_src,
            body,
        })
    }

    fn parse_headers(&mut self) -> Result<IndexMap<String, String>> {
        let mut map = IndexMap::new();
        loop {
            match self.peek() {
                None => break,
                Some((lineno, content)) => {
                    let t = content.trim();
                    if t == "---" || t.is_empty() || t.starts_with("//") {
                        break;
                    }
                    if let Some(colon) = t.find(':') {
                        let key = t[..colon].trim().to_owned();
                        let val = t[colon + 1..].trim().to_owned();
                        map.insert(key, val);
                        self.advance();
                    } else {
                        return Err(self.err(lineno, format!("invalid header line: `{t}`")));
                    }
                }
            }
        }
        Ok(map)
    }

    fn expect_body_start(&mut self) -> Result<()> {
        match self.advance() {
            Some((_, l)) if l.trim() == "---" => Ok(()),
            Some((n, l)) => Err(self.err(n, format!("expected `---`, got `{}`", l.trim()))),
            None => Err(self.err(self.pos, "unexpected end of file, expected `---`")),
        }
    }

    fn expect_node_end(&mut self) -> Result<()> {
        loop {
            match self.peek() {
                None => return Err(self.err(self.pos, "unexpected end of file, expected `===`")),
                Some((_, l)) if l.trim().is_empty() || l.trim().starts_with("//") => {
                    self.advance();
                }
                Some((_, l)) if l.trim() == "===" => {
                    self.advance();
                    return Ok(());
                }
                Some((n, l)) => {
                    return Err(self.err(n, format!("expected `===`, got `{}`", l.trim())));
                }
            }
        }
    }
}

// ── body parsing (delegates individual stmts to stmt module) ──────────────────

impl Parser<'_> {
    /// Parses body statements at or deeper than `min_indent`.
    pub(super) fn parse_body(&mut self, min_indent: usize) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                None => break,
                Some((_, content)) => {
                    let indent = leading_spaces(content);
                    let t = content.trim();
                    if t.is_empty() || t.starts_with("//") {
                        self.advance();
                        continue;
                    }
                    if t == "===" {
                        break;
                    }
                    if indent < min_indent {
                        break;
                    }
                    if matches!(t, "<<else>>" | "<<elseif" | "<<endif>>" | "<<endonce>>")
                        || t.starts_with("<<elseif ")
                    {
                        break;
                    }
                    let stmt = self.parse_stmt(min_indent)?;
                    stmts.push(stmt);
                }
            }
        }
        Ok(stmts)
    }

    pub(super) fn parse_stmt(&mut self, cur_indent: usize) -> Result<Stmt> {
        let (lineno, content) = self.peek().unwrap();
        let t = content.trim();

        if t.starts_with("<<") {
            return self.parse_command_stmt(lineno, cur_indent);
        }
        if t.starts_with("->") {
            return self.parse_option_block(cur_indent);
        }
        if t.starts_with("=>") {
            return Ok(self.parse_line_group(cur_indent));
        }
        self.advance();
        Ok(parse_line_stmt(t))
    }

    // ── shortcut options ──────────────────────────────────────────────────────

    pub(super) fn parse_option_block(&mut self, cur_indent: usize) -> Result<Stmt> {
        let mut items = Vec::new();
        while let Some((_, content)) = self.peek() {
            let t = content.trim();
            if !t.starts_with("->") {
                break;
            }
            let option_indent = leading_spaces(content);
            if option_indent < cur_indent {
                break;
            }
            self.advance();
            let rest = t[2..].trim();
            let (text_part, cond_src, once) = parse_option_text(rest);
            let (text, tags) = split_trailing_tags(&text_part);
            let id = self.next_id();
            let body = self.parse_body(option_indent + 1)?;
            items.push(OptionItem {
                id,
                text,
                cond_src,
                once,
                tags,
                body,
            });
        }
        Ok(Stmt::Options(items))
    }

    // ── line groups ───────────────────────────────────────────────────────────

    pub(super) fn parse_line_group(&mut self, cur_indent: usize) -> Stmt {
        let mut variants = Vec::new();
        while let Some((_, content)) = self.peek() {
            let t = content.trim();
            if !t.starts_with("=>") {
                break;
            }
            if leading_spaces(content) < cur_indent {
                break;
            }
            self.advance();
            let rest = t[2..].trim();
            let id = self.next_id();
            let (line_text, cond_src, once) = parse_option_text(rest);
            let (speaker, text) = helpers::split_speaker(&line_text);
            let (text, tags) = split_trailing_tags(&text);
            variants.push(LineVariant {
                id,
                speaker,
                text,
                cond_src,
                once,
                tags,
            });
        }
        Stmt::LineGroup(variants)
    }
}
