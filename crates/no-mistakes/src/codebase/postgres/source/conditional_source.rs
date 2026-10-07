//! Conditional ranges use the prepared parser's delimiter ownership.
use sqlparser::{
    ast::{Spanned, Statement},
    tokenizer::{Location, Token, TokenWithSpan},
};
use std::ops::RangeInclusive;

pub(super) fn statement_range(
    statement: &Statement,
    tokens: &[&TokenWithSpan],
    cursor: Location,
) -> Result<RangeInclusive<usize>, String> {
    let first = tokens.partition_point(|token| token.span.start < cursor);
    let start = first
        + tokens[first..]
            .iter()
            .position(|token| !matches!(token.token, Token::Whitespace(_)))
            .ok_or("Conditional statement source start is unavailable")?;
    // IF's closing token belongs to this exact AST node, even with nested IF,
    // CASE expressions, quoted keywords and comments. Its condition span alone
    // cannot prove the enclosing END IF boundary.
    let end = match statement {
        Statement::If(value) => {
            value
                .end_token
                .as_ref()
                .ok_or("Conditional closing IF source token is unavailable")?
                .0
                .span
                .end
        }
        _ => statement.span().end,
    };
    // Some supported parser statements (notably LOCK) have an empty AST span.
    // Never let that rewind the source cursor into a preceding statement.
    let search = tokens
        .partition_point(|token| token.span.end < end)
        .max(start);
    let finish = search
        + tokens[search..]
            .iter()
            .position(|token| token.token == Token::SemiColon)
            .ok_or("Conditional statement delimiter is unavailable")?;
    Ok(start..=finish)
}
