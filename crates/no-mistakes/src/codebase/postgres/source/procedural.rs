//! Procedural occurrences are source facts, never proof that a body executes.
use super::body::Body;
use super::locations::Locations;
use super::procedural_prepare::{prepare_block, PreparedBlock};
use super::types::*;
use crate::codebase::postgres::parse::PreparedPostgresTokens;
use sqlparser::{
    keywords::Keyword,
    parser::Parser,
    tokenizer::{Location, Token, TokenWithSpan},
};

pub(super) fn starts(parser: &Parser<'_>) -> bool {
    matches!(parser.peek_token().token, Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::DO)
}

pub(super) fn collect(
    parser: &mut Parser<'_>,
    source: &PostgresSqlSource,
    locations: &Locations<'_>,
    depth: usize,
) -> Result<PostgresSqlStatementKind, String> {
    match prepare_block(parser, source, locations, depth)? {
        PreparedBlock::Done(block) => Ok(PostgresSqlStatementKind::DoBlock { block }),
        PreparedBlock::Parse {
            block,
            prepared,
            body,
            retain_walker,
        } => parse_nested(
            block,
            prepared,
            &body,
            source,
            locations,
            depth,
            retain_walker,
        ),
    }
}

#[inline(never)]
fn parse_nested(
    mut block: PostgresSqlProceduralBlock,
    mut prepared: PreparedPostgresTokens,
    body: &Body<'_>,
    source: &PostgresSqlSource,
    locations: &Locations<'_>,
    depth: usize,
    retain_walker: bool,
) -> Result<PostgresSqlStatementKind, String> {
    let sql = body.sql.as_ref();
    let local = Locations::new(sql);
    let mut recursive_views = crate::codebase::postgres::parse::RecursiveViews::default();
    for token in &mut prepared.tokens {
        let local_start = token.span.start;
        relocate(token, &local, locations, body)?;
        prepared
            .recursive_views
            .relocate(local_start, token.span.start, &mut recursive_views);
    }
    prepared.recursive_views = recursive_views;
    let nested = super::parsing::collect_program(source, prepared, locations, depth + 1, true);
    let header_diagnostics = std::mem::take(&mut block.diagnostics);
    block.statements = nested.statements;
    block.diagnostics = nested.diagnostics;
    block.diagnostics.extend(header_diagnostics);
    super::procedural_walk::finish_parsed(&mut block, retain_walker);
    Ok(PostgresSqlStatementKind::DoBlock { block })
}

pub(super) fn relocate(
    token: &mut TokenWithSpan,
    local: &Locations<'_>,
    original: &Locations<'_>,
    body: &Body<'_>,
) -> Result<(), String> {
    let span = local
        .span(token.span)
        .ok_or("DO nested token span is unavailable")?;
    let start = body
        .offset(span.start.offset)
        .ok_or("DO nested token start is unavailable")?;
    let end = body
        .offset(span.end.offset)
        .ok_or("DO nested token end is unavailable")?;
    let span = original.range(start, end);
    token.span.start = Location {
        line: span.start.line as u64,
        column: span.start.column as u64,
    };
    token.span.end = Location {
        line: span.end.line as u64,
        column: span.end.column as u64,
    };
    Ok(())
}
