use super::*;

fn tokens(src: &str) -> Vec<Token> {
    tokenise(src, "<test>", 0)
        .expect("tokenise failed in test")
        .into_iter()
        .map(|(t, _)| t)
        .collect()
}

#[test]
fn lex_integer() {
    assert_eq!(tokens("42"), vec![Token::Number(42.0)]);
}

#[test]
fn lex_float() {
    assert_eq!(tokens("2.5"), vec![Token::Number(2.5)]);
}

#[test]
fn lex_string_literal() {
    assert_eq!(tokens(r#""hello""#), vec![Token::Str("hello".into())]);
}

#[test]
fn lex_variable() {
    assert_eq!(tokens("$gold"), vec![Token::Var("$gold".into())]);
}

#[test]
fn lex_comparison_tokens() {
    assert_eq!(
        tokens(">= <= > < == !="),
        vec![
            Token::Gte,
            Token::Lte,
            Token::Gt,
            Token::Lt,
            Token::EqEq,
            Token::Neq
        ]
    );
}

#[test]
fn lex_arithmetic_tokens() {
    assert_eq!(
        tokens("+ - * / %"),
        vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Percent
        ]
    );
}

#[test]
fn lex_logical_tokens() {
    assert_eq!(
        tokens("&& || !"),
        vec![Token::AndAnd, Token::OrOr, Token::Bang]
    );
}

#[test]
fn lex_ident_vs_var() {
    let toks = tokens("hello $world");
    assert_eq!(toks[0], Token::Ident("hello".into()));
    assert_eq!(toks[1], Token::Var("$world".into()));
}

#[test]
fn lex_cmd_delimiters() {
    assert_eq!(tokens("<< >>"), vec![Token::CmdOpen, Token::CmdClose]);
}

#[test]
fn lex_node_delimiters() {
    assert_eq!(tokens("--- ==="), vec![Token::BodyStart, Token::NodeEnd]);
}
