//! `statement.span()` and `Query::span()` both end at sqlparser's partial span.
//! `DO NOTHING` and `EXISTS` omit their closing syntax. The CTE closing `)` is
//! the next token, so a nested statement is every token before that parenthesis.

use super::super::locations::Locations;
use super::super::types::{PostgresSqlQuery, PostgresSqlQueryClause, PostgresSqlQueryUnsupported};
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, TokenWithSpan, Tokenizer};

const BOUNDARY: &str = "nested data-modifying statement boundary";

pub(super) fn repair(
    facts: &mut PostgresSqlQuery,
    locations: &Locations<'_>,
    scope: usize,
    closing: &AttachedToken,
) {
    let indexes = facts
        .nested_statements
        .iter()
        .enumerate()
        .filter(|(_, statement)| statement.query_scope_id == scope)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for index in indexes {
        repair_statement(facts, locations, scope, closing, index);
    }
}

fn repair_statement(
    facts: &mut PostgresSqlQuery,
    locations: &Locations<'_>,
    scope: usize,
    closing: &AttachedToken,
    index: usize,
) {
    let Some(current) = facts.nested_statements[index].span.clone() else {
        return;
    };
    let Some(boundary) = closing_offset(closing, locations) else {
        reject(facts, index, scope);
        return;
    };
    if current.start.offset > boundary || current.end.offset > boundary {
        reject(facts, index, scope);
        return;
    }
    let tail = locations.slice(&locations.range(current.end.offset, boundary));
    let extra = match significant_end(tail) {
        Ok(end) => end.unwrap_or(0),
        Err(()) => {
            reject(facts, index, scope);
            return;
        }
    };
    if extra == 0 {
        return;
    }
    let mut span = current;
    let end = span.end.offset + extra;
    span.end = locations.range(end, end).start;
    let sql = locations.slice(&span).to_owned();
    let statement = &mut facts.nested_statements[index];
    statement.span = Some(span);
    statement.sql = sql;
}

fn closing_offset(closing: &AttachedToken, locations: &Locations<'_>) -> Option<usize> {
    if closing.0.token != Token::RParen {
        return None;
    }
    Some(locations.position(closing.0.span.start)?.offset)
}

fn reject(facts: &mut PostgresSqlQuery, index: usize, scope: usize) {
    let statement = &mut facts.nested_statements[index];
    let partial = statement.span.clone();
    statement.sql.clear();
    statement.span = None;
    statement.complete = false;
    let item = PostgresSqlQueryUnsupported {
        scope_id: scope,
        clause: PostgresSqlQueryClause::Other,
        reason: BOUNDARY.into(),
        span: partial,
    };
    statement.unsupported.push(item.clone());
    facts.unsupported.push(item);
    facts.complete = false;
}

/// Byte offset of the last non-trivia token in `tail`, relative to `tail`.
/// `Ok(None)` means the tail is empty or only whitespace and comments.
fn significant_end(tail: &str) -> Result<Option<usize>, ()> {
    if tail.is_empty() {
        return Ok(None);
    }
    let mut tokens = Vec::new();
    if Tokenizer::new(&PostgreSqlDialect {}, tail)
        .tokenize_with_location_into_buf(&mut tokens)
        .is_err()
    {
        return Err(());
    }
    boundary_end(tail, &tokens)
}

/// `Err` when `tokens` do not consume `tail` or a significant token has no
/// source position. Callers then drop the truncated slice instead of guessing.
fn boundary_end(tail: &str, tokens: &[TokenWithSpan]) -> Result<Option<usize>, ()> {
    let local = Locations::new(tail);
    let last = tokens.last().ok_or(())?;
    let consumed = local
        .position(last.span.end)
        .map(|position| position.offset);
    if consumed != Some(tail.len()) {
        return Err(());
    }
    let mut end = None;
    for token in tokens {
        if matches!(token.token, Token::Whitespace(_) | Token::EOF) {
            continue;
        }
        end = Some(local.position(token.span.end).ok_or(())?.offset);
    }
    Ok(end)
}

#[cfg(test)]
mod tests;
