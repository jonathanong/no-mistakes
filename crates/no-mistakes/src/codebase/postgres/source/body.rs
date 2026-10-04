//! Map a distinct DO program back to its original literal without reparsing SQL.
use super::types::{PostgresSqlBodyEncoding, PostgresSqlSpan};
use sqlparser::tokenizer::Token;
use std::borrow::Cow;

pub(super) struct Body<'a> {
    pub sql: Cow<'a, str>,
    pub start: usize,
    pub end: usize,
    pub encoding: PostgresSqlBodyEncoding,
    escaped_quotes: Vec<usize>,
}

impl Body<'_> {
    pub(super) fn offset(&self, decoded: usize) -> Option<usize> {
        if decoded > self.sql.len() {
            return None;
        }
        let extra = self
            .escaped_quotes
            .partition_point(|after| *after <= decoded);
        Some(self.start + decoded + extra)
    }
}

pub(super) fn decode<'a>(
    token: &Token,
    span: &PostgresSqlSpan,
    source: &'a str,
) -> Result<Body<'a>, String> {
    let (delimiter, expected, encoding) = match token {
        Token::DollarQuotedString(body) => (
            body.tag.as_ref().map_or(2, |tag| tag.len() + 2),
            body.value.as_str(),
            PostgresSqlBodyEncoding::DollarQuoted,
        ),
        Token::SingleQuotedString(body) => {
            (1, body.as_str(), PostgresSqlBodyEncoding::SingleQuoted)
        }
        _ => {
            return Err(
                "DO source facts require a dollar-quoted or standard single-quoted body".into(),
            )
        }
    };
    let start = span.start.offset + delimiter;
    let end = span
        .end
        .offset
        .checked_sub(delimiter)
        .ok_or("DO body source span is unavailable")?;
    let raw = source
        .get(start..end)
        .ok_or("DO body source span is unavailable")?;
    let (sql, escaped_quotes) = if encoding == PostgresSqlBodyEncoding::SingleQuoted {
        let mut decoded = String::new();
        let mut escaped_quotes = Vec::new();
        let mut at = 0;
        while at < raw.len() {
            let value = raw[at..].chars().next().unwrap();
            let width = value.len_utf8();
            let consumed = if value == '\'' && raw[at..].starts_with("''") {
                2
            } else {
                width
            };
            decoded.push(value);
            if consumed > width {
                escaped_quotes.push(decoded.len());
            }
            at += consumed;
        }
        (Cow::Owned(decoded), escaped_quotes)
    } else {
        (Cow::Borrowed(raw), Vec::new())
    };
    if sql != expected {
        return Err("DO body spelling cannot be mapped to original source".into());
    }
    Ok(Body {
        sql,
        start,
        end,
        encoding,
        escaped_quotes,
    })
}
