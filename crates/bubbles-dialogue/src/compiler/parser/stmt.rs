//! Command, if, and once statement parsing - `impl Parser` blocks for control flow.

use crate::compiler::ast::{IfBranch, Stmt};
use crate::error::Result;

use super::assignments::{parse_declare, parse_expr_arc, parse_interpolated, parse_set};
use super::command::{extract_cmd, extract_cmd_line_tags, split_first_word};
use super::text::split_trailing_tags;
use super::{Parser, into_stmt_list};

impl Parser<'_> {
    pub(super) fn parse_command_stmt(&mut self, lineno: usize, cur_indent: usize) -> Result<Stmt> {
        let (_, content) = self
            .advance()
            .ok_or_else(|| self.err(lineno, "unexpected end of input"))?;
        let t = content.trim();
        let (t_core, line_tags) = extract_cmd_line_tags(t);
        let inner = extract_cmd(t_core, lineno, self.file)?;

        let (kw, rest) = split_first_word(inner);
        match kw {
            "jump" => Ok(Stmt::Jump(rest.trim().to_owned())),
            "detour" => Ok(Stmt::Detour(rest.trim().to_owned())),
            "return" => Ok(Stmt::Return),
            "stop" => Ok(Stmt::Stop),
            "set" => parse_set(inner, lineno, self.file),
            "declare" => parse_declare(inner, lineno, self.file),
            "if" => self.parse_if(inner, lineno, cur_indent),
            "once" => self.parse_once(inner, lineno, cur_indent),
            _ => {
                let (args_raw, inner_tags) = split_trailing_tags(rest);
                let args = parse_interpolated(&args_raw, "command args", lineno, self.file)?;
                let mut tags = inner_tags;
                tags.extend(line_tags);
                Ok(Stmt::Command {
                    name: kw.to_owned(),
                    args,
                    tags,
                })
            }
        }
    }

    fn parse_if(&mut self, first_cond: &str, if_lineno: usize, cur_indent: usize) -> Result<Stmt> {
        let cond_src = first_cond[2..].trim();
        let cond0 = parse_expr_arc(cond_src, "<<if>>", if_lineno, self.file)?;
        let body = self.parse_body(cur_indent + 1)?;
        let mut branches: Vec<IfBranch> = vec![IfBranch {
            cond: cond0,
            body: into_stmt_list(body),
        }];
        let mut else_body: Vec<Stmt> = Vec::new();

        loop {
            match self.peek() {
                Some((_, l)) if l.trim().starts_with("<<elseif ") => {
                    let last = self.last_lineno();
                    let (lineno2, content) = self
                        .advance()
                        .ok_or_else(|| self.err(last, "unexpected end of input"))?;
                    let inner = extract_cmd(content.trim(), lineno2, self.file)?;
                    let cond_src2 = inner["elseif".len()..].trim();
                    let cond2 = parse_expr_arc(cond_src2, "<<elseif>>", lineno2, self.file)?;
                    let b = self.parse_body(cur_indent + 1)?;
                    branches.push(IfBranch {
                        cond: cond2,
                        body: into_stmt_list(b),
                    });
                }
                Some((_, l)) if l.trim() == "<<else>>" => {
                    self.advance();
                    else_body = self.parse_body(cur_indent + 1)?;
                    break;
                }
                _ => break,
            }
        }
        match self.peek() {
            Some((_, l)) if l.trim() == "<<endif>>" => {
                self.advance();
            }
            Some((line, l)) => {
                return Err(self.err(line, format!("expected `<<endif>>`, got `{}`", l.trim())));
            }
            None => {
                return Err(self.err(if_lineno, "unexpected end of file, expected `<<endif>>`"));
            }
        }
        Ok(Stmt::If {
            branches,
            else_body: into_stmt_list(else_body),
        })
    }

    fn parse_once(&mut self, inner: &str, once_lineno: usize, cur_indent: usize) -> Result<Stmt> {
        let block_id = self.next_id();
        // `inner` is the full command text after `<<`, e.g. `once`, `once if expr`.
        let rest = inner["once".len()..].trim();
        let cond = if rest.starts_with("if ") || rest == "if" {
            let src = rest["if".len()..].trim();
            Some(parse_expr_arc(src, "<<once if>>", once_lineno, self.file)?)
        } else {
            None
        };
        let body = self.parse_body(cur_indent + 1)?;
        let else_body = if let Some((_, l)) = self.peek()
            && l.trim() == "<<else>>"
        {
            self.advance();
            self.parse_body(cur_indent + 1)?
        } else {
            Vec::new()
        };
        match self.peek() {
            Some((_, l)) if l.trim() == "<<endonce>>" => {
                self.advance();
            }
            Some((line, l)) => {
                return Err(self.err(line, format!("expected `<<endonce>>`, got `{}`", l.trim())));
            }
            None => {
                return Err(self.err(
                    once_lineno,
                    "unexpected end of file, expected `<<endonce>>`",
                ));
            }
        }
        Ok(Stmt::Once {
            block_id,
            cond,
            body: into_stmt_list(body),
            else_body: into_stmt_list(else_body),
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
