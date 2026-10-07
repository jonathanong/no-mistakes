//! COMMENT literals borrow the original source and prepared parser.
use super::super::locations::Locations;
use sqlparser::{keywords::Keyword, parser::Parser, tokenizer::Token};

pub(super) fn text(
    parser: &mut Parser<'_>,
    locations: &Locations<'_>,
) -> Result<Option<String>, String> {
    let token = parser.next_token();
    let source_span = locations
        .span(token.span)
        .ok_or("COMMENT literal source span is unavailable")?;
    let source = locations.slice(&source_span);
    // Decode original spelling, never an already-decoded tokenizer value.
    if source
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("u&'"))
    {
        let value = source[3..]
            .strip_suffix('\'')
            .ok_or("Unterminated Unicode COMMENT string")?;
        let escape = if parser.parse_keyword(Keyword::UESCAPE) {
            let Token::SingleQuotedString(value) = parser.next_token().token else {
                return Err("UESCAPE requires a single-quoted escape character".into());
            };
            let mut chars = value.chars();
            let escape = chars
                .next()
                .ok_or("UESCAPE requires one escape character")?;
            if chars.next().is_some()
                || escape.is_ascii_hexdigit()
                || escape.is_whitespace()
                || matches!(escape, '+' | '\'' | '"')
            {
                return Err("Invalid UESCAPE character".into());
            }
            escape
        } else {
            '\\'
        };
        let decoded =
            crate::codebase::postgres::parse::unicode::decode_unicode_string(value, escape)
                .filter(|value| !value.contains('\0'))
                .ok_or("Invalid Unicode COMMENT escape sequence")?;
        return Ok(Some(decoded));
    }
    comment_value(&token.token)
}

/// Non-Unicode COMMENT values use the already-prepared literal token.
fn comment_value(token: &Token) -> Result<Option<String>, String> {
    match token {
        Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::NULL => {
            Ok(None)
        }
        Token::SingleQuotedString(value) | Token::EscapedStringLiteral(value) => {
            Ok(Some(value.clone()))
        }
        Token::DollarQuotedString(value) => Ok(Some(value.value.clone())),
        _ => Err("Expected a single-quoted COMMENT string or NULL".into()),
    }
}
