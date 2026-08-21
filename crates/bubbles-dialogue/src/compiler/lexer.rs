//! Logos-based lexer that tokenises `.bub` source and expression strings.

use logos::Logos;

/// Resolves the `\"`, `\\`, and `\n` escapes in a string-literal body in a
/// single pass. Unknown escapes are kept verbatim.
fn unescape(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('\\') | None => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
        }
    }
    out
}

/// A lexical token produced by the lexer.
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
        Some(unescape(&s[1..s.len() - 1]))
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

/// Lexes `input` into a [`Vec`] of spanned tokens, returning an error on
/// any character that does not match a known token.
///
/// # Errors
///
/// Returns [`crate::error::DialogueError::Parse`] with `file` / `line` context
/// when an unrecognised character is encountered, so the caller receives a
/// precise pointer into the source rather than a confusing downstream failure.
pub fn tokenise(input: &str, file: &str, line: usize) -> crate::error::Result<Vec<Spanned>> {
    let mut tokens = Vec::new();
    for (result, span) in Token::lexer(input).spanned() {
        if let Ok(tok) = result {
            tokens.push((tok, span));
        } else {
            let ch = input[span].chars().next().unwrap_or('?');
            return Err(crate::error::DialogueError::Parse {
                file: file.to_owned(),
                line,
                message: format!(
                    "unexpected character `{ch}` in expression; \
                     did you mean `$` for a variable?"
                ),
            });
        }
    }
    Ok(tokens)
}

#[cfg(test)]
#[path = "lexer_tests.rs"]
mod tests;
