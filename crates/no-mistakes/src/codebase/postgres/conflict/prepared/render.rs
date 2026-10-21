//! sqlparser `Display` writes the unescaped value, so `'it''s'` becomes `'it's'`.
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};
use std::fmt::Write;

pub(in crate::codebase::postgres::conflict) fn render_chunk(tokens: &[TokenWithSpan]) -> String {
    let mut text = String::new();
    for token in tokens {
        render_token(&token.token, &mut text);
    }
    text
}

fn render_token(token: &Token, text: &mut String) {
    match token {
        Token::EOF => {}
        Token::SingleQuotedString(value) => push_quoted(text, '\'', value),
        Token::NationalStringLiteral(value) => {
            text.push('N');
            push_quoted(text, '\'', value);
        }
        Token::EscapedStringLiteral(value) => push_escaped_literal(text, value),
        Token::Word(Word {
            quote_style: Some(quote),
            value,
            ..
        }) => push_quoted(text, *quote, value),
        other => {
            let _ = write!(text, "{other}");
        }
    }
}

fn push_quoted(text: &mut String, quote: char, value: &str) {
    text.push(quote);
    for character in value.chars() {
        text.push(character);
        if character == quote {
            text.push(quote);
        }
    }
    text.push(quote);
}

/// `E'...'` unescapes `\\` and quotes. Double both so the token round-trips.
/// A backslash quote would hide `ON CONFLICT` from the conflict lexer.
fn push_escaped_literal(text: &mut String, value: &str) {
    text.push('E');
    text.push('\'');
    for character in value.chars() {
        match character {
            '\\' => text.push_str("\\\\"),
            '\'' => text.push_str("''"),
            other => text.push(other),
        }
    }
    text.push('\'');
}
