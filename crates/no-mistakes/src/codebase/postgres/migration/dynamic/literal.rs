mod provenance;

use super::*;
use provenance::literal_source_bytes;
use std::borrow::Cow;

#[derive(Debug)]
pub(super) struct DecodedString {
    pub(super) sql: String,
    pub(super) source_bytes: Vec<usize>,
    pub(super) dollar_quoted: bool,
}

pub(super) fn string_expression(
    tokens: &[&TokenWithSpan],
    source: Option<&str>,
) -> Option<DecodedString> {
    let (tokens, unicode_escape) = split_string_expression(tokens)?;
    let mut sql = String::new();
    let mut source_bytes = Vec::new();
    let mut previous = None;
    let dollar_quoted = tokens.len() == 1
        && tokens
            .first()
            .is_some_and(|token| matches!(token.token, Token::DollarQuotedString(_)));
    for token in tokens {
        let value = string_token(&token.token, unicode_escape)?;
        if let Some(previous_end_line) = previous {
            if token.span.start.line <= previous_end_line {
                return None;
            }
        }
        source_bytes.extend(
            source
                .and_then(|source| literal_source_bytes(source, token, &value, unicode_escape))
                .unwrap_or_else(|| vec![token.span.start.line as usize; value.len()]),
        );
        sql.push_str(&value);
        previous = Some(token.span.end.line);
    }
    (!source_bytes.is_empty()).then_some(DecodedString {
        sql,
        source_bytes,
        dollar_quoted,
    })
}

pub(super) fn leading_string_expression(
    tokens: &[&TokenWithSpan],
    source: Option<&str>,
) -> Option<DecodedString> {
    let start = tokens
        .iter()
        .position(|token| raw_string_token(&token.token).is_some())?;
    let literal_end = tokens[start..]
        .iter()
        .position(|token| raw_string_token(&token.token).is_none())
        .map_or(tokens.len(), |count| start + count);
    let end = if unicode_escape_character(&tokens[literal_end..]).is_some() {
        literal_end + 2
    } else {
        literal_end
    };
    string_expression(&tokens[start..end], source)
}

fn split_string_expression<'a>(
    tokens: &'a [&'a TokenWithSpan],
) -> Option<(&'a [&'a TokenWithSpan], Option<char>)> {
    let literal_end = tokens
        .iter()
        .position(|token| raw_string_token(&token.token).is_none())
        .unwrap_or(tokens.len());
    if literal_end == 0 {
        return None;
    }
    let suffix = &tokens[literal_end..];
    if suffix.is_empty() {
        return Some((tokens, None));
    }
    let escape = unicode_escape_character(suffix)?;
    matches!(
        tokens[literal_end - 1].token,
        Token::UnicodeStringLiteral(_)
    )
    .then_some((&tokens[..literal_end], Some(escape)))
}

fn unicode_escape_character(tokens: &[&TokenWithSpan]) -> Option<char> {
    if tokens.len() != 2 || !word(tokens[0], "UESCAPE") {
        return None;
    }
    let Token::SingleQuotedString(value) = &tokens[1].token else {
        return None;
    };
    let mut chars = value.chars();
    let escape = chars.next()?;
    (chars.next().is_none()
        && !escape.is_ascii_hexdigit()
        && !matches!(escape, '+' | '\'' | '"')
        && !escape.is_whitespace())
    .then_some(escape)
}

fn string_token(token: &Token, unicode_escape: Option<char>) -> Option<Cow<'_, str>> {
    match token {
        Token::UnicodeStringLiteral(value) => {
            decode_unicode_string(value, unicode_escape.unwrap_or('\\')).map(Cow::Owned)
        }
        _ => raw_string_token(token).map(Cow::Borrowed),
    }
}

fn raw_string_token(token: &Token) -> Option<&str> {
    match token {
        Token::SingleQuotedString(value)
        | Token::EscapedStringLiteral(value)
        | Token::UnicodeStringLiteral(value) => Some(value),
        Token::DollarQuotedString(value) => Some(&value.value),
        _ => None,
    }
}

pub(super) fn decode_unicode_string(value: &str, escape: char) -> Option<String> {
    super::super::super::parse::unicode::decode_unicode_string(value, escape)
}

pub(super) fn normalize_format(template: &str) -> String {
    let mut out = String::new();
    let mut chars = template.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '%' {
            out.push(character);
            continue;
        }
        if chars.next_if_eq(&'%').is_some() {
            out.push('%');
            continue;
        }
        let mut directive = String::new();
        let mut replaced = false;
        while let Some(next) = chars.peek().copied() {
            if matches!(next, 'I' | 'L' | 's') {
                chars.next();
                out.push_str(match next {
                    'I' => "dynamic_identifier",
                    'L' => "'dynamic_literal'",
                    's' => "dynamic_value",
                    _ => unreachable!("format type was matched above"),
                });
                replaced = true;
                break;
            }
            if next.is_ascii_digit() || matches!(next, '$' | '-' | '*') {
                directive.push(next);
                chars.next();
                continue;
            }
            break;
        }
        if !replaced {
            out.push('%');
            out.push_str(&directive);
        }
    }
    out
}
