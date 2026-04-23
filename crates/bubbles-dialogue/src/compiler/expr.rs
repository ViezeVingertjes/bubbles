//! Recursive-descent expression parser - turns a source string into an [`Expr`] tree.

pub use crate::compiler::ast::{BinOp, Expr, UnOp};

use crate::compiler::lexer::{Token, tokenise};
use crate::error::{DialogueError, Result};

// ── public entry point ────────────────────────────────────────────────────────

/// Parses `source` as an expression, with no surrounding source context.
///
/// Errors surface with `file = "<expr>"` and `line = 0`.  Callers inside the
/// compiler should prefer [`parse_expr_at`] so parse failures point at the
/// real `.bub` line.
///
/// # Errors
/// Returns [`DialogueError::Parse`] on a syntax error.
pub fn parse_expr(source: &str) -> Result<Expr> {
    parse_expr_at(source, "<expr>", 0)
}

/// Parses `source` as an expression, tagging any resulting parse error with
/// `file` and `line` so the reported location matches the enclosing `.bub`
/// statement.
///
/// # Errors
/// Returns [`DialogueError::Parse`] on a syntax error.
pub fn parse_expr_at(source: &str, file: &str, line: usize) -> Result<Expr> {
    let tokens: Vec<Token> = tokenise(source).into_iter().map(|(t, _)| t).collect();
    let mut p = ExprParser {
        tokens: &tokens,
        pos: 0,
        file,
        line,
    };
    let expr = p.parse_or()?;
    if p.pos < p.tokens.len() {
        return Err(DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("unexpected token after expression: {:?}", p.tokens[p.pos]),
        });
    }
    Ok(expr)
}

// ── parser ────────────────────────────────────────────────────────────────────

struct ExprParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    file: &'a str,
    line: usize,
}

impl ExprParser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn err(&self, msg: &str) -> DialogueError {
        DialogueError::Parse {
            file: self.file.to_owned(),
            line: self.line,
            message: msg.into(),
        }
    }

    // Precedence (lowest → highest):
    // or → and → eq/neq → cmp → add/sub → mul/div/rem → unary → primary

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::OrOr) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while self.peek() == Some(&Token::AndAnd) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => BinOp::Eq,
                Some(Token::Neq) => BinOp::Neq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::Lte) => BinOp::Lte,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::Gte) => BinOp::Gte,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Rem,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Some(Token::Bang) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.advance().cloned() {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Str(s)) => Ok(Expr::Text(s)),
            Some(Token::Var(v)) => Ok(Expr::Var(v)),
            Some(Token::Ident(ref s)) if s == "true" => Ok(Expr::Bool(true)),
            Some(Token::Ident(ref s)) if s == "false" => Ok(Expr::Bool(false)),
            Some(Token::Ident(name)) => {
                if self.peek() == Some(&Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_or()?);
                        while self.peek() == Some(&Token::Comma) {
                            self.advance();
                            args.push(self.parse_or()?);
                        }
                    }
                    if self.advance() != Some(&Token::RParen) {
                        return Err(self.err("expected `)` after function arguments"));
                    }
                    Ok(Expr::Call { name, args })
                } else {
                    Err(self.err(&format!(
                        "unknown identifier `{name}`; variables need a `$` prefix"
                    )))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_or()?;
                if self.advance() != Some(&Token::RParen) {
                    return Err(self.err("expected closing `)`"));
                }
                Ok(expr)
            }
            Some(t) => Err(self.err(&format!("unexpected token `{t:?}`"))),
            None => Err(self.err("unexpected end of expression")),
        }
    }
}

#[cfg(test)]
#[path = "expr_tests.rs"]
mod tests;
