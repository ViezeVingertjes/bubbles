//! Command, if, and once statement parsing — `impl Parser` blocks for control flow.

use crate::compiler::ast::Stmt;
use crate::error::Result;

use super::Parser;
use super::helpers::{
    extract_cmd, extract_cmd_line_tags, first_word, parse_declare, parse_set, split_first_word,
    split_trailing_tags,
};

impl Parser<'_> {
    pub(super) fn parse_command_stmt(&mut self, lineno: usize, cur_indent: usize) -> Result<Stmt> {
        let (_, content) = self.advance().unwrap();
        let t = content.trim();
        let (t_core, line_tags) = extract_cmd_line_tags(t);
        let inner = extract_cmd(t_core, lineno, self.file)?;

        let kw = first_word(inner);
        match kw {
            "jump" => Ok(Stmt::Jump(inner[kw.len()..].trim().to_owned())),
            "detour" => Ok(Stmt::Detour(inner[kw.len()..].trim().to_owned())),
            "return" => Ok(Stmt::Return),
            "set" => parse_set(inner, lineno, self.file),
            "declare" => parse_declare(inner, lineno, self.file),
            "if" => self.parse_if(inner, cur_indent),
            "once" => self.parse_once(inner, cur_indent),
            _ => {
                let (cmd_name, rest) = split_first_word(inner);
                let (args_src, inner_tags) = split_trailing_tags(rest);
                let mut tags = inner_tags;
                tags.extend(line_tags);
                Ok(Stmt::Command {
                    name: cmd_name.to_owned(),
                    args_src,
                    tags,
                })
            }
        }
    }

    fn parse_if(&mut self, first_cond: &str, cur_indent: usize) -> Result<Stmt> {
        // The raw line we already advanced past is no longer in the buffer, so
        // we look at the *previous* pos for the line number. Use a conservative
        // approach: report errors at the current pos (next unread line) which is
        // one past the <<if>> line.
        let if_lineno = self.pos.saturating_sub(1);
        let cond = first_cond[2..].trim().to_owned();
        // Validate the condition expression eagerly so the error points at the
        // <<if>> line, not at some later <<endif>> or ===.
        crate::compiler::expr::parse_expr(&cond).map_err(|_| {
            self.err(
                if_lineno,
                format!("invalid expression in `<<if>>`: `{cond}`"),
            )
        })?;
        let body = self.parse_body(cur_indent + 1)?;
        let mut branches: Vec<(String, Vec<Stmt>)> = vec![(cond, body)];
        let mut else_body = Vec::new();

        loop {
            match self.peek() {
                Some((_, l)) if l.trim().starts_with("<<elseif ") => {
                    let (lineno2, content) = self.advance().unwrap();
                    let inner = extract_cmd(content.trim(), lineno2, self.file)?;
                    let cond2 = inner["elseif".len()..].trim().to_owned();
                    crate::compiler::expr::parse_expr(&cond2).map_err(|_| {
                        self.err(
                            lineno2,
                            format!("invalid expression in `<<elseif>>`: `{cond2}`"),
                        )
                    })?;
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
        if let Some((_, l)) = self.peek() {
            if l.trim() == "<<endif>>" {
                self.advance();
            }
        }
        Ok(Stmt::If {
            branches,
            else_body,
        })
    }

    fn parse_once(&mut self, inner: &str, cur_indent: usize) -> Result<Stmt> {
        let block_id = self.next_id();
        let once_lineno = self.pos.saturating_sub(1);
        // `inner` is the full command text after `<<`, e.g. `once`, `once if expr`.
        let rest = inner["once".len()..].trim();
        let cond_src = if rest.starts_with("if ") || rest == "if" {
            let src = rest["if".len()..].trim().to_owned();
            crate::compiler::expr::parse_expr(&src).map_err(|_| {
                self.err(
                    once_lineno,
                    format!("invalid expression in `<<once if>>`: `{src}`"),
                )
            })?;
            Some(src)
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
        Ok(Stmt::Once {
            block_id,
            cond_src,
            body,
            else_body,
        })
    }
}

// ── shared helpers ────────────────────────────────────────────────────────────

/// Splits raw option/variant text into `(text, cond_src, once)`.
pub(super) fn parse_option_text(s: &str) -> (String, Option<String>, bool) {
    let once = s.starts_with("once ");
    let s = if once { s["once ".len()..].trim() } else { s };

    if let Some(idx) = s.rfind("<<if ") {
        let text = s[..idx].trim().to_owned();
        let cond_src = s[idx + 5..].trim_end_matches(">>").trim().to_owned();
        return (text, Some(cond_src), once);
    }
    (s.to_owned(), None, once)
}
