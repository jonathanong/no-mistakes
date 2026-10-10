//! Separate the DO envelope from nested parsing so recursion stays stack-bounded.
use super::body::Body;
use super::locations::Locations;
use super::procedural_omit::sql_occurrence;
use super::types::*;
use crate::codebase::postgres::parse::PreparedPostgresTokens;
use sqlparser::{keywords::Keyword, parser::Parser, tokenizer::Token};

pub(super) enum PreparedBlock<'a> {
    Done(PostgresSqlProceduralBlock),
    Parse {
        block: PostgresSqlProceduralBlock,
        prepared: PreparedPostgresTokens,
        body: Body<'a>,
        retain_walker: bool,
    },
}

#[inline(never)]
pub(super) fn prepare_block<'a>(
    parser: &mut Parser<'_>,
    source: &'a PostgresSqlSource,
    locations: &Locations<'_>,
    depth: usize,
) -> Result<PreparedBlock<'a>, String> {
    parser.next_token();
    let mut language = "plpgsql".to_string();
    let before = parser.parse_keyword(Keyword::LANGUAGE);
    if before {
        language = language_name(parser)?;
    }
    if matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
        return Err("Expected a DO body".into());
    }
    let literal = parser.next_token();
    if parser.parse_keyword(Keyword::LANGUAGE) {
        if before {
            return Err("DO language may be specified only once".into());
        }
        language = language_name(parser)?;
    }
    let literal_span = locations
        .span(literal.span)
        .ok_or("DO body source span is unavailable")?;
    let body = super::body::decode(&literal.token, &literal_span, &source.sql)?;
    let body_span = locations.range(body.start, body.end);
    let mut block = PostgresSqlProceduralBlock {
        language,
        body_encoding: body.encoding,
        body_span: body_span.clone(),
        statements: Vec::new(),
        occurrences: Vec::new(),
        diagnostics: Vec::new(),
        complete: false,
    };
    if depth >= 64 {
        block.diagnostics.push(diagnostic(
            "Procedural source nesting exceeds the parser safety limit",
            &body_span,
        ));
        return Ok(PreparedBlock::Done(block));
    }
    if block.language != "plpgsql" {
        block.diagnostics.push(diagnostic(
            "Only plain plpgsql DO blocks have structured occurrence facts",
            &body_span,
        ));
        return Ok(PreparedBlock::Done(block));
    }
    let sql = body.sql.as_ref();
    let mut prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(sql);
    if let Some(error) = &prepared.lexical_error {
        // A lexical failure prevents proving the surrounding procedural grammar.
        block
            .diagnostics
            .push(diagnostic(&error.to_string(), &body_span));
        return Ok(PreparedBlock::Done(block));
    }
    // `prepared.tokens` includes parser rewrites. Classify the pre-rewrite inventory.
    let walked = super::procedural_walk::walk(&prepared.occurrence_tokens, &body, locations);
    block.occurrences = walked.occurrences;
    if walked.legacy_stop {
        block.diagnostics.push(diagnostic(
            "Procedural control flow is unsupported; no nested DDL execution or occurrence is inferred",
            &body_span,
        ));
        return Ok(PreparedBlock::Done(block));
    }
    if walked.walker_only && !occurrences_include_sql(&block.occurrences) {
        super::procedural_walk::finish_unparsed(&mut block, &body_span);
        return Ok(PreparedBlock::Done(block));
    }
    let significant = prepared
        .tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_) | Token::SemiColon)).then_some(index)
        })
        .collect::<Vec<_>>();
    let plain = significant
        .first()
        .is_some_and(|index| keyword(&prepared.tokens[*index].token, Keyword::BEGIN))
        && significant
            .last()
            .is_some_and(|index| keyword(&prepared.tokens[*index].token, Keyword::END));
    if !plain {
        block.diagnostics.push(diagnostic(
            "Procedural control flow is unsupported; no nested DDL execution or occurrence is inferred",
            &body_span,
        ));
        return Ok(PreparedBlock::Done(block));
    }
    let first = significant[0];
    let last = *significant.last().unwrap();
    prepared.tokens = prepared
        .tokens
        .into_iter()
        .enumerate()
        .filter_map(|(index, token)| (index > first && index < last).then_some(token))
        .collect();
    if walked.walker_only {
        // Occurrence spans are original-source offsets. These tokens are still body-local.
        super::procedural_omit::omit_non_sql(&mut prepared.tokens, &body, &block.occurrences);
    }
    super::conditional::prepare(&mut prepared.tokens);
    Ok(PreparedBlock::Parse {
        block,
        prepared,
        body,
        retain_walker: walked.walker_only,
    })
}

fn language_name(parser: &mut Parser<'_>) -> Result<String, String> {
    if matches!(parser.peek_token().token, Token::SemiColon | Token::EOF) {
        return Err("Expected a DO language identifier".into());
    }
    match parser.next_token().token {
        Token::Word(word) => Ok(if word.quote_style.is_some() {
            word.value
        } else {
            word.value.to_ascii_lowercase()
        }),
        _ => Err("Expected a DO language identifier".into()),
    }
}

fn keyword(token: &Token, value: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == value)
}

fn occurrences_include_sql(occurrences: &[PostgresSqlProceduralOccurrence]) -> bool {
    occurrences
        .iter()
        .any(|occurrence| sql_occurrence(occurrence.kind))
}

fn diagnostic(message: &str, span: &PostgresSqlSpan) -> PostgresSqlDiagnostic {
    PostgresSqlDiagnostic {
        message: message.into(),
        span: Some(span.clone()),
    }
}
