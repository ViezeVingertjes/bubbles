//! Logos-based lexer that tokenises `.bub` source and expression strings.

use logos::Logos;

/// A lexical token produced by the lexer.
#[allow(clippy::upper_case_acronyms)]
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\f]+")] // skip horizontal whitespace; newlines are significant in the parser
pub enum Token {
    // ── literals ──────────────────────────────────────────────────────────────
    /// Floating-point or integer literal.
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),

    /// Double-quoted string literal.
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].replace("\\\"", "\"").replace("\\\\", "\\").replace("\\n", "\n"))
    })]
    Str(String),

    // ── identifiers / keywords ─────────────────────────────────────────────────
    /// Variable beginning with `$`.
    #[regex(r"\$[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_owned())]
    Var(String),

    /// Plain identifier or keyword.
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),

    // ── delimiters ─────────────────────────────────────────────────────────────
    /// `(` – opens a parenthesised sub-expression or argument list.
    #[token("(")]
    LParen,
    /// `)` – closes a parenthesised sub-expression or argument list.
    #[token(")")]
    RParen,
    /// `,` – argument separator.
    #[token(",")]
    Comma,
    /// `<<` – opens a command/statement block.
    #[token("<<")]
    CmdOpen,
    /// `>>` – closes a command/statement block.
    #[token(">>")]
    CmdClose,
    /// `{` – opens an inline expression.
    #[token("{")]
    BraceOpen,
    /// `}` – closes an inline expression.
    #[token("}")]
    BraceClose,

    // ── arithmetic ─────────────────────────────────────────────────────────────
    /// `+`
    #[token("+")]
    Plus,
    /// `-`
    #[token("-")]
    Minus,
    /// `*`
    #[token("*")]
    Star,
    /// `/`
    #[token("/")]
    Slash,
    /// `%`
    #[token("%")]
    Percent,

    // ── comparison (order matters: `>=` before `>`) ───────────────────────────
    /// `>=`
    #[token(">=")]
    Gte,
    /// `<=`
    #[token("<=")]
    Lte,
    /// `>`
    #[token(">")]
    Gt,
    /// `<`
    #[token("<")]
    Lt,
    /// `==`
    #[token("==")]
    EqEq,
    /// `!=`
    #[token("!=")]
    Neq,

    // ── logical ────────────────────────────────────────────────────────────────
    /// `&&`
    #[token("&&")]
    AndAnd,
    /// `||`
    #[token("||")]
    OrOr,
    /// `!`
    #[token("!")]
    Bang,

    // ── assignment / misc ──────────────────────────────────────────────────────
    /// `=` (used in `<<set $x = …>>`)
    #[token("=")]
    Eq,
    /// `:`
    #[token(":")]
    Colon,
    /// `->`
    #[token("->")]
    Arrow,
    /// `=>`
    #[token("=>")]
    FatArrow,
    /// `---` body-start delimiter.
    #[token("---")]
    BodyStart,
    /// `===` node-end delimiter.
    #[token("===")]
    NodeEnd,
    /// `#` tag prefix.
    #[token("#")]
    Hash,
    /// Newline.
    #[token("\n")]
    Newline,
}

/// A spanned token pair.
pub type Spanned = (Token, std::ops::Range<usize>);

/// Lexes `input` into a [`Vec`] of spanned tokens, silently skipping errors.
///
/// This function is used by the expression parser and by tests.
#[must_use]
pub fn tokenise(input: &str) -> Vec<Spanned> {
    Token::lexer(input)
        .spanned()
        .filter_map(|(t, span)| t.ok().map(|t| (t, span)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(src: &str) -> Vec<Token> {
        tokenise(src).into_iter().map(|(t, _)| t).collect()
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
}
