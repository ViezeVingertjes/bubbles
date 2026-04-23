//! Unit tests for the expression parser (`super::parse_expr_at`).

use super::{BinOp, Expr, UnOp, parse_expr_at};

fn p(src: &str) -> Result<Expr, crate::error::DialogueError> {
    parse_expr_at(src, "<test>", 0)
}

fn num(n: f64) -> Expr {
    Expr::Number(n)
}
fn bin(left: Expr, op: BinOp, right: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

#[test]
fn parse_literal_number() {
    assert_eq!(p("42").unwrap(), num(42.0));
}

#[test]
fn parse_literal_bool() {
    assert_eq!(p("true").unwrap(), Expr::Bool(true));
    assert_eq!(p("false").unwrap(), Expr::Bool(false));
}

#[test]
fn parse_addition() {
    assert_eq!(p("1 + 2").unwrap(), bin(num(1.0), BinOp::Add, num(2.0)));
}

#[test]
fn mul_has_higher_precedence_than_add() {
    let ast = p("1 + 2 * 3").unwrap();
    let expected = bin(num(1.0), BinOp::Add, bin(num(2.0), BinOp::Mul, num(3.0)));
    assert_eq!(ast, expected);
}

#[test]
fn parentheses_override_precedence() {
    let ast = p("(1 + 2) * 3").unwrap();
    let expected = bin(bin(num(1.0), BinOp::Add, num(2.0)), BinOp::Mul, num(3.0));
    assert_eq!(ast, expected);
}

#[test]
fn unary_negation() {
    assert_eq!(
        p("-5").unwrap(),
        Expr::Unary {
            op: UnOp::Neg,
            expr: Box::new(num(5.0))
        }
    );
}

#[test]
fn logical_not() {
    assert_eq!(
        p("!true").unwrap(),
        Expr::Unary {
            op: UnOp::Not,
            expr: Box::new(Expr::Bool(true))
        }
    );
}

#[test]
fn comparison_chain() {
    assert_eq!(p("3 > 2").unwrap(), bin(num(3.0), BinOp::Gt, num(2.0)));
}

#[test]
fn function_call_no_args() {
    assert_eq!(
        p("random()").unwrap(),
        Expr::Call {
            name: "random".into(),
            args: vec![]
        }
    );
}

#[test]
fn function_call_with_args() {
    assert_eq!(
        p("dice(6, 2)").unwrap(),
        Expr::Call {
            name: "dice".into(),
            args: vec![num(6.0), num(2.0)]
        }
    );
}

#[test]
fn parse_rejects_bare_identifier_without_dollar() {
    assert!(p("not_a_var").is_err());
}

#[test]
fn parse_rejects_unterminated_expression() {
    assert!(p("3 *").is_err());
}

#[test]
fn parse_rejects_unclosed_parenthesis() {
    assert!(p("(1 + 2").is_err());
}

#[test]
fn parse_function_three_args() {
    let ast = p("clamp(1, 2, 3)").unwrap();
    let Expr::Call { name, args } = ast else {
        panic!("expected call");
    };
    assert_eq!(name, "clamp");
    assert_eq!(args.len(), 3);
}
