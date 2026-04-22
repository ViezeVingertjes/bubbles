//! Line-oriented recursive-descent parser that converts `.bub` source into a [`Vec<Node>`].

pub(super) mod assignments;
pub(super) mod body;
pub(super) mod command;
pub(super) mod stmt;
pub(super) mod text;

use indexmap::IndexMap;

use crate::compiler::ast::Node;
use crate::error::{DialogueError, Result};

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

    /// Returns the source line number that `advance()` last consumed, or the
    /// file's final source line if no lines have been consumed yet. Used for
    /// reporting errors that fire after end-of-file.
    pub(super) fn last_lineno(&self) -> usize {
        if self.pos == 0 {
            self.lines.first().map_or(1, |&(n, _)| n)
        } else {
            self.lines
                .get(self.pos - 1)
                .or_else(|| self.lines.last())
                .map_or(1, |&(n, _)| n)
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
                line: self.last_lineno(),
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
                    // `===` in header position means the author forgot `---`.
                    if t == "===" {
                        return Err(self.err(
                            lineno,
                            "found `===` where `---` body delimiter was expected — \
                             is the `---` line missing?",
                        ));
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
            None => Err(self.err(self.last_lineno(), "unexpected end of file, expected `---`")),
        }
    }

    fn expect_node_end(&mut self) -> Result<()> {
        loop {
            match self.peek() {
                None => {
                    return Err(
                        self.err(self.last_lineno(), "unexpected end of file, expected `===`")
                    );
                }
                Some((_, l)) if l.trim().is_empty() || l.trim().starts_with("//") => {
                    self.advance();
                }
                Some((_, l)) if l.trim() == "===" => {
                    self.advance();
                    return Ok(());
                }
                Some((n, l)) => {
                    let t = l.trim();
                    let hint = if matches!(t, "<<endif>>" | "<<endonce>>") {
                        format!(
                            " — `{t}` has no matching opening block; \
                             check indentation and that every `<<if>>` or `<<once>>` \
                             has a corresponding `{t}`"
                        )
                    } else if t == "<<else>>" || t.starts_with("<<elseif") {
                        " — unexpected `<<else>>`/`<<elseif>>`; \
                         check that the matching `<<if>>` is correct"
                            .to_owned()
                    } else {
                        String::new()
                    };
                    return Err(self.err(n, format!("expected `===`, got `{t}`{hint}")));
                }
            }
        }
    }
}

// Body-level parsing lives in [`body`].
