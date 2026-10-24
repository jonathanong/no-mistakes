//! A generated bind must be a parameter token, not identifier/string/comment text.
use super::SqlText;
use sqlparser::{
    dialect::PostgreSqlDialect,
    tokenizer::{Token, Tokenizer},
};
use std::collections::BTreeSet;

pub(super) fn valid(text: &SqlText) -> bool {
    if text.binds.is_empty() {
        return true;
    }
    let Ok(tokens) = Tokenizer::new(&PostgreSqlDialect {}, &text.raw.sql).tokenize_with_location()
    else {
        return false;
    };
    let positions = tokens
        .into_iter()
        .filter_map(|token| {
            matches!(token.token, Token::Placeholder(_)).then_some((
                token.span.start.line,
                token.span.start.column,
                token.span.end.line,
                token.span.end.column,
            ))
        })
        .collect::<BTreeSet<_>>();
    text.binds.iter().all(|&(start, end)| {
        let Some((start_line, start_column)) = super::coordinate(&text.raw.sql, start) else {
            return false;
        };
        let Some((end_line, end_column)) = super::coordinate(&text.raw.sql, end) else {
            return false;
        };
        positions.contains(&(
            start_line as u64,
            start_column as u64,
            end_line as u64,
            end_column as u64,
        ))
    })
}
