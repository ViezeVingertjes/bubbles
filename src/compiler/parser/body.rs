//! Body-level parsing: statement dispatch, option blocks, and line groups.
//!
//! Individual command statements (`<<…>>`) are delegated to [`super::stmt`].

use crate::compiler::ast::{LineVariant, OptionItem, Stmt};
use crate::error::Result;

use super::Parser;
use super::assignments::{parse_expr_arc, parse_interpolated};
use super::stmt::parse_option_text;
use super::text::{leading_spaces, parse_line_stmt, split_speaker, split_trailing_tags};

impl Parser<'_> {
    /// Parses body statements at or deeper than `min_indent`.
    pub(super) fn parse_body(&mut self, min_indent: usize) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while let Some((lineno, content)) = self.peek() {
            let indent = leading_spaces(content);
            let t = content.trim();
            if t.is_empty() || t.starts_with("//") {
                self.advance();
                continue;
            }
            if t == "===" {
                break;
            }
            // A `title:` line inside a body almost certainly means the
            // author forgot `===` to close the previous node.
            if min_indent == 0 && t.starts_with("title:") && t.len() > "title:".len() {
                return Err(self.err(
                    lineno,
                    "found `title:` inside a node body — \
                     did you forget `===` to close the previous node?",
                ));
            }
            if indent < min_indent {
                break;
            }
            if matches!(t, "<<else>>" | "<<endif>>" | "<<endonce>>") || t.starts_with("<<elseif ") {
                break;
            }
            stmts.push(self.parse_stmt(min_indent)?);
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
            return self.parse_line_group(cur_indent);
        }
        let (lineno, _) = self.advance().unwrap();
        let file = self.file;
        parse_line_stmt(t, lineno, file)
    }

    pub(super) fn parse_option_block(&mut self, cur_indent: usize) -> Result<Stmt> {
        let mut items = Vec::new();
        let file = self.file;
        while let Some((lineno, content)) = self.peek() {
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
            let (text_part, cond_str, once) = parse_option_text(rest);
            let cond = match cond_str {
                None => None,
                Some(s) => Some(parse_expr_arc(
                    &s,
                    "shortcut option `<<if>>`",
                    lineno,
                    file,
                )?),
            };
            let (raw, tags) = split_trailing_tags(&text_part);
            let text = parse_interpolated(&raw, "option text", lineno, file)?;
            let id = self.next_id();
            let body = self.parse_body(option_indent + 1)?;
            items.push(OptionItem {
                id,
                text,
                cond,
                once,
                tags,
                body,
            });
        }
        Ok(Stmt::Options(items))
    }

    pub(super) fn parse_line_group(&mut self, cur_indent: usize) -> Result<Stmt> {
        let mut variants = Vec::new();
        let file = self.file;
        while let Some((lineno, content)) = self.peek() {
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
            let (line_text, cond_str, once) = parse_option_text(rest);
            let cond = match cond_str {
                None => None,
                Some(s) => Some(parse_expr_arc(&s, "line group `<<if>>`", lineno, file)?),
            };
            let (speaker, raw) = split_speaker(&line_text);
            let (raw, tags) = split_trailing_tags(&raw);
            let text = parse_interpolated(&raw, "line group text", lineno, file)?;
            variants.push(LineVariant {
                id,
                speaker,
                text,
                cond,
                once,
                tags,
            });
        }
        Ok(Stmt::LineGroup(variants))
    }
}
