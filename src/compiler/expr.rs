//! Expression AST and recursive-descent parser.

use crate::compiler::lexer::{Token, tokenise};
use crate::error::{DialogueError, Result};

/// A node in the expression AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal.
    Number(f64),
    /// String literal.
    Text(String),
    /// Boolean literal.
    Bool(bool),
    /// Variable read, e.g. `$gold`.
    Var(String),
    /// Function call, e.g. `random(1, 6)`.
    Call {
        /// Function name.
        name: String,
        /// Argument expressions.
        args: Vec<Expr>,
    },
    /// Unary operator.
    Unary {
        /// Operator.
        op: UnOp,
        /// Operand.
        expr: Box<Expr>,
    },
    /// Binary operator.
    Binary {
        /// Left operand.
        left: Box<Expr>,
        /// Operator.
        op: BinOp,
        /// Right operand.
        right: Box<Expr>,
    },
}

/// Binary operator kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
    /// `==`
    Eq,
    /// `!=`
    Neq,
    /// `<`
    Lt,
    /// `<=`
    Lte,
    /// `>`
    Gt,
    /// `>=`
    Gte,
    /// `&&`
    And,
    /// `||`
    Or,
}

/// Unary operator kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Arithmetic negation `-`.
    Neg,
    /// Logical negation `!`.
    Not,
}

// ── public entry point ────────────────────────────────────────────────────────

/// Parses `source` as an expression.
///
/// # Errors
/// Returns [`DialogueError::Parse`] on a syntax error.
pub fn parse_expr(source: &str) -> Result<Expr> {
    let tokens: Vec<Token> = tokenise(source).into_iter().map(|(t, _)| t).collect();
    let mut p = ExprParser { tokens: &tokens, pos: 0 };
    let expr = p.parse_or()?;
    if p.pos < p.tokens.len() {
        return Err(DialogueError::Parse {
            file: "<expr>".into(),
            line: 0,
            message: format!("unexpected token after expression: {:?}", p.tokens[p.pos]),
        });
    }
    Ok(expr)
}

// ── parser ────────────────────────────────────────────────────────────────────

struct ExprParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> ExprParser<'a> {
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
            file: "<expr>".into(),
            line: 0,
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
            left = Expr::Binary { left: Box::new(left), op: BinOp::Or, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while self.peek() == Some(&Token::AndAnd) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::And, right: Box::new(right) };
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
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
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
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
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
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
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
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnOp::Neg, expr: Box::new(expr) })
            }
            Some(Token::Bang) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnOp::Not, expr: Box::new(expr) })
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
                // could be a function call
                if self.peek() == Some(&Token::LParen) {
                    self.advance(); // consume `(`
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
                    Err(self.err(&format!("unknown identifier `{name}`; variables need a `$` prefix")))
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
mod tests {
    use super::*;

    fn num(n: f64) -> Expr { Expr::Number(n) }
    fn bin(left: Expr, op: BinOp, right: Expr) -> Expr {
        Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
    }

    #[test]
    fn parse_literal_number() {
        assert_eq!(parse_expr("42").unwrap(), num(42.0));
    }

    #[test]
    fn parse_literal_bool() {
        assert_eq!(parse_expr("true").unwrap(), Expr::Bool(true));
        assert_eq!(parse_expr("false").unwrap(), Expr::Bool(false));
    }

    #[test]
    fn parse_addition() {
        assert_eq!(
            parse_expr("1 + 2").unwrap(),
            bin(num(1.0), BinOp::Add, num(2.0))
        );
    }

    #[test]
    fn mul_has_higher_precedence_than_add() {
        // 1 + 2 * 3  →  1 + (2 * 3)
        let ast = parse_expr("1 + 2 * 3").unwrap();
        let expected = bin(num(1.0), BinOp::Add, bin(num(2.0), BinOp::Mul, num(3.0)));
        assert_eq!(ast, expected);
    }

    #[test]
    fn parentheses_override_precedence() {
        // (1 + 2) * 3
        let ast = parse_expr("(1 + 2) * 3").unwrap();
        let expected = bin(bin(num(1.0), BinOp::Add, num(2.0)), BinOp::Mul, num(3.0));
        assert_eq!(ast, expected);
    }

    #[test]
    fn unary_negation() {
        assert_eq!(
            parse_expr("-5").unwrap(),
            Expr::Unary { op: UnOp::Neg, expr: Box::new(num(5.0)) }
        );
    }

    #[test]
    fn logical_not() {
        assert_eq!(
            parse_expr("!true").unwrap(),
            Expr::Unary { op: UnOp::Not, expr: Box::new(Expr::Bool(true)) }
        );
    }

    #[test]
    fn comparison_chain() {
        assert_eq!(
            parse_expr("3 > 2").unwrap(),
            bin(num(3.0), BinOp::Gt, num(2.0))
        );
    }

    #[test]
    fn function_call_no_args() {
        assert_eq!(
            parse_expr("random()").unwrap(),
            Expr::Call { name: "random".into(), args: vec![] }
        );
    }

    #[test]
    fn function_call_with_args() {
        assert_eq!(
            parse_expr("dice(6, 2)").unwrap(),
            Expr::Call { name: "dice".into(), args: vec![num(6.0), num(2.0)] }
        );
    }
}
