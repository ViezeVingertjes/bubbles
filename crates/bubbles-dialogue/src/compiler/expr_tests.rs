//! Unit tests for the expression parser (`super::parse_expr`).

use super::{BinOp, Expr, UnOp, parse_expr};

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
    let ast = parse_expr("1 + 2 * 3").unwrap();
    let expected = bin(num(1.0), BinOp::Add, bin(num(2.0), BinOp::Mul, num(3.0)));
    assert_eq!(ast, expected);
}

#[test]
fn parentheses_override_precedence() {
    let ast = parse_expr("(1 + 2) * 3").unwrap();
    let expected = bin(bin(num(1.0), BinOp::Add, num(2.0)), BinOp::Mul, num(3.0));
    assert_eq!(ast, expected);
}

#[test]
fn unary_negation() {
    assert_eq!(
        parse_expr("-5").unwrap(),
        Expr::Unary {
            op: UnOp::Neg,
            expr: Box::new(num(5.0))
        }
    );
}

#[test]
fn logical_not() {
    assert_eq!(
        parse_expr("!true").unwrap(),
        Expr::Unary {
            op: UnOp::Not,
            expr: Box::new(Expr::Bool(true))
        }
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
        Expr::Call {
            name: "random".into(),
            args: vec![]
        }
    );
}

#[test]
fn function_call_with_args() {
    assert_eq!(
        parse_expr("dice(6, 2)").unwrap(),
        Expr::Call {
            name: "dice".into(),
            args: vec![num(6.0), num(2.0)]
        }
    );
}

#[test]
fn parse_rejects_bare_identifier_without_dollar() {
    assert!(parse_expr("not_a_var").is_err());
}

#[test]
fn parse_rejects_unterminated_expression() {
    assert!(parse_expr("3 *").is_err());
}

#[test]
fn parse_rejects_unclosed_parenthesis() {
    assert!(parse_expr("(1 + 2").is_err());
}

#[test]
fn parse_function_three_args() {
    let ast = parse_expr("clamp(1, 2, 3)").unwrap();
    let Expr::Call { name, args } = ast else {
        panic!("expected call");
    };
    assert_eq!(name, "clamp");
    assert_eq!(args.len(), 3);
}
