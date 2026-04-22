//! Line-oriented recursive-descent parser that converts `.bub` source into a [`Vec<Node>`].

use indexmap::IndexMap;

use crate::compiler::ast::{LineVariant, Node, OptionItem, Stmt};
use crate::error::{DialogueError, Result};

// ── public entry point ────────────────────────────────────────────────────────

/// Parses a single `.bub` source string into a list of [`Node`]s.
pub fn parse(file: &str, source: &str) -> Result<Vec<Node>> {
    let mut p = Parser::new(file, source);
    p.parse_file()
}

// ── parser state ──────────────────────────────────────────────────────────────

struct Parser<'src> {
    file: &'src str,
    lines: Vec<(usize, &'src str)>, // (1-based line number, content)
    pos: usize,
    /// counter for generating stable unique block ids
    id_counter: usize,
}

impl<'src> Parser<'src> {
    fn new(file: &'src str, source: &'src str) -> Self {
        let lines = source
            .lines()
            .enumerate()
            .map(|(i, l)| (i + 1, l))
            .collect();
        Self { file, lines, pos: 0, id_counter: 0 }
    }

    fn next_id(&mut self) -> String {
        self.id_counter += 1;
        format!("blk{}", self.id_counter)
    }

    fn peek(&self) -> Option<(usize, &'src str)> {
        self.lines.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<(usize, &'src str)> {
        let line = self.lines.get(self.pos).copied();
        self.pos += 1;
        line
    }

    fn err(&self, line: usize, msg: impl Into<String>) -> DialogueError {
        DialogueError::Parse {
            file: self.file.to_owned(),
            line,
            message: msg.into(),
        }
    }

    /// Consume lines that are blank or comment-only.
    fn skip_blank_and_comments(&mut self) {
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

impl<'src> Parser<'src> {
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

        Ok(Node { title, tags, headers: extra, when_src, body })
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
        // peek past blank lines
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

// ── body / statement parsing ──────────────────────────────────────────────────

impl<'src> Parser<'src> {
    /// Parses body statements at or deeper than `min_indent`.
    /// Stops when encountering a line at a shallower indent, `===`, or EOF.
    fn parse_body(&mut self, min_indent: usize) -> Result<Vec<Stmt>> {
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
                    // stop when the indentation returns to the caller's level
                    if indent < min_indent {
                        break;
                    }
                    // stop on known "end" keywords at any depth
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

    fn parse_stmt(&mut self, cur_indent: usize) -> Result<Stmt> {
        let (lineno, content) = self.peek().unwrap();
        let t = content.trim();

        if t.starts_with("<<") {
            return self.parse_command_stmt(lineno, cur_indent);
        }
        if t.starts_with("->") {
            return self.parse_option_block(cur_indent);
        }
        if t.starts_with("=>") {
            return self.parse_line_group(cur_indent);
        }
        // plain line
        self.advance();
        Ok(parse_line_stmt(t))
    }

    // ── commands ──────────────────────────────────────────────────────────────

    fn parse_command_stmt(&mut self, lineno: usize, cur_indent: usize) -> Result<Stmt> {
        let (_, content) = self.advance().unwrap();
        let t = content.trim();
        let inner = extract_cmd(t, lineno, self.file)?;

        let kw = first_word(inner);
        match kw {
            "jump" => {
                let target = inner[kw.len()..].trim().to_owned();
                Ok(Stmt::Jump(target))
            }
            "detour" => {
                let target = inner[kw.len()..].trim().to_owned();
                Ok(Stmt::Detour(target))
            }
            "return" => Ok(Stmt::Return),
            "set" => parse_set(inner, lineno, self.file),
            "declare" => parse_declare(inner, lineno, self.file),
            "if" => self.parse_if(inner, cur_indent, lineno),
            "once" => self.parse_once(inner, cur_indent, lineno),
            _ => {
                let (cmd_name, rest) = split_first_word(inner);
                let (args_src, tags) = split_trailing_tags(rest);
                Ok(Stmt::Command {
                    name: cmd_name.to_owned(),
                    args_src: args_src.to_owned(),
                    tags,
                })
            }
        }
    }

    // ── if / elseif / else / endif ────────────────────────────────────────────

    fn parse_if(&mut self, first_cond: &str, cur_indent: usize, _lineno: usize) -> Result<Stmt> {
        let cond = first_cond[2..].trim().to_owned(); // strip "if"
        let body = self.parse_body(cur_indent + 1)?;
        let mut branches: Vec<(String, Vec<Stmt>)> = vec![(cond, body)];
        let mut else_body = Vec::new();

        loop {
            match self.peek() {
                Some((_, l)) if l.trim().starts_with("<<elseif ") => {
                    let (lineno2, content) = self.advance().unwrap();
                    let inner = extract_cmd(content.trim(), lineno2, self.file)?;
                    let cond2 = inner["elseif".len()..].trim().to_owned();
                    let b = self.parse_body(cur_indent + 1)?;
                    branches.push((cond2, b));
                }
                Some((_, l)) if l.trim() == "<<else>>" => {
                    self.advance();
                    else_body = self.parse_body(cur_indent + 1)?;
                    break;
                }
                _ => break,
            }
        }
        // consume <<endif>>
        if let Some((_, l)) = self.peek() {
            if l.trim() == "<<endif>>" {
                self.advance();
            }
        }
        Ok(Stmt::If { branches, else_body })
    }

    // ── once ──────────────────────────────────────────────────────────────────

    fn parse_once(&mut self, rest: &str, cur_indent: usize, _lineno: usize) -> Result<Stmt> {
        let block_id = self.next_id();
        let cond_src = if rest.starts_with("if ") || rest == "if" {
            Some(rest["if".len()..].trim().to_owned())
        } else {
            None
        };
        let body = self.parse_body(cur_indent + 1)?;
        let mut else_body = Vec::new();
        if let Some((_, l)) = self.peek() {
            if l.trim() == "<<else>>" {
                self.advance();
                else_body = self.parse_body(cur_indent + 1)?;
            }
        }
        if let Some((_, l)) = self.peek() {
            if l.trim() == "<<endonce>>" {
                self.advance();
            }
        }
        Ok(Stmt::Once { block_id, cond_src, body, else_body })
    }

    // ── shortcut options ──────────────────────────────────────────────────────

    fn parse_option_block(&mut self, cur_indent: usize) -> Result<Stmt> {
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
            // body is indented one more than the `->` line
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

    fn parse_line_group(&mut self, cur_indent: usize) -> Result<Stmt> {
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
            let (speaker, text) = split_speaker(&line_text);
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
        Ok(Stmt::LineGroup(variants))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn leading_spaces(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

fn extract_cmd<'a>(t: &'a str, line: usize, file: &str) -> Result<&'a str> {
    let inner = t
        .strip_prefix("<<")
        .and_then(|s| s.strip_suffix(">>"))
        .ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("malformed command `{t}`"),
        })?;
    Ok(inner.trim())
}

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

fn split_first_word(s: &str) -> (&str, &str) {
    match s.find(|c: char| c.is_ascii_whitespace()) {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}

fn parse_set(inner: &str, line: usize, file: &str) -> Result<Stmt> {
    // inner = "set $var = expr" or "set $var to expr"
    let rest = inner["set".len()..].trim();
    let (name, after) = split_first_word(rest);
    let expr_src = after
        .strip_prefix('=')
        .or_else(|| after.strip_prefix("to "))
        .map(str::trim)
        .ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("expected `= expr` or `to expr` after variable in `<<set>>`, got `{after}`"),
        })?
        .to_owned();
    Ok(Stmt::Set { name: name.to_owned(), expr_src })
}

fn parse_declare(inner: &str, line: usize, file: &str) -> Result<Stmt> {
    // inner = "declare $var = expr"
    let rest = inner["declare".len()..].trim();
    let (name, after) = split_first_word(rest);
    let expr_src = after
        .strip_prefix('=')
        .map(str::trim)
        .ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("expected `= expr` after variable in `<<declare>>`, got `{after}`"),
        })?
        .to_owned();
    Ok(Stmt::Declare { name: name.to_owned(), expr_src })
}

/// Splits raw option/variant text into `(text, cond_src, once)`.
fn parse_option_text(s: &str) -> (String, Option<String>, bool) {
    let once = s.starts_with("once ");
    let s = if once { s["once ".len()..].trim() } else { s };

    if let Some(idx) = s.rfind("<<if ") {
        let text = s[..idx].trim().to_owned();
        let cond_src = s[idx + 5..]
            .trim_end_matches(">>")
            .trim()
            .to_owned();
        return (text, Some(cond_src), once);
    }
    (s.to_owned(), None, once)
}

/// Splits a line into `(speaker, text)` if it looks like `Name: text`.
fn split_speaker(s: &str) -> (Option<String>, String) {
    if let Some(colon) = s.find(':') {
        let candidate = &s[..colon];
        // speaker names don't contain spaces and must not look like a URL
        if !candidate.contains(' ') && !candidate.is_empty() && colon + 1 < s.len() {
            let text = s[colon + 1..].trim().to_owned();
            return (Some(candidate.trim().to_owned()), text);
        }
    }
    (None, s.to_owned())
}

/// Extracts trailing ` #tag` tokens from the end of a string.
fn split_trailing_tags(s: &str) -> (String, Vec<String>) {
    let mut text = s.trim_end().to_owned();
    let mut tags = Vec::new();
    loop {
        // look for a tag at the very end: ` #word`
        let trimmed = text.trim_end();
        if let Some(hash_pos) = trimmed.rfind(" #") {
            let tag_candidate = &trimmed[hash_pos + 2..];
            if !tag_candidate.is_empty() && !tag_candidate.contains(' ') {
                tags.push(tag_candidate.to_owned());
                text = trimmed[..hash_pos].trim_end().to_owned();
                continue;
            }
        }
        break;
    }
    tags.reverse();
    (text, tags)
}

fn parse_line_stmt(t: &str) -> Stmt {
    let (speaker, rest) = split_speaker(t);
    let (text, tags) = split_trailing_tags(&rest);
    Stmt::Line { speaker, text, tags }
}
