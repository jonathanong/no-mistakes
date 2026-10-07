//! Procedural occurrences are source facts, never proof that a body executes.
use super::{locations::Locations, types::*};
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
    let start = body.start;
    let end = body.end;
    let body_span = locations.range(start, end);
    let mut block = PostgresSqlProceduralBlock {
        language,
        body_encoding: body.encoding,
        body_span: body_span.clone(),
        statements: Vec::new(),
        diagnostics: Vec::new(),
        complete: false,
    };
    if depth >= 64 {
        block.diagnostics.push(diagnostic(
            "Procedural source nesting exceeds the parser safety limit",
            &body_span,
        ));
        return Ok(PostgresSqlStatementKind::DoBlock { block });
    }
    if block.language != "plpgsql" {
        block.diagnostics.push(diagnostic(
            "Only plain plpgsql DO blocks have structured occurrence facts",
            &body_span,
        ));
        return Ok(PostgresSqlStatementKind::DoBlock { block });
    }
    let sql = body.sql.as_ref();
    let mut prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(sql);
    if let Some(error) = &prepared.lexical_error {
        // A lexical failure prevents proving the surrounding procedural grammar.
        block
            .diagnostics
            .push(diagnostic(&error.to_string(), &body_span));
        return Ok(PostgresSqlStatementKind::DoBlock { block });
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
    let control = significant.iter().any(|index| matches!(&prepared.tokens[*index].token, Token::Word(word) if word.quote_style.is_none() && ["LOOP", "EXCEPTION", "DECLARE"].iter().any(|value| word.value.eq_ignore_ascii_case(value))));
    if !plain || control {
        block.diagnostics.push(diagnostic("Procedural control flow is unsupported; no nested DDL execution or occurrence is inferred", &body_span));
        return Ok(PostgresSqlStatementKind::DoBlock { block });
    }
    let first = significant[0];
    let last = *significant.last().unwrap();
    prepared.tokens = prepared
        .tokens
        .into_iter()
        .enumerate()
        .filter_map(|(index, token)| (index > first && index < last).then_some(token))
        .collect();
    super::conditional::prepare(&mut prepared.tokens);
    let local = Locations::new(sql);
    let mut recursive_views = crate::codebase::postgres::parse::RecursiveViews::default();
    for token in &mut prepared.tokens {
        let local_start = token.span.start;
        relocate(token, &local, locations, &body)?;
        prepared
            .recursive_views
            .relocate(local_start, token.span.start, &mut recursive_views);
    }
    prepared.recursive_views = recursive_views;
    let nested = super::parsing::collect_program(source, prepared, locations, depth + 1);
    block.complete = nested.diagnostics.is_empty()
        && nested
            .statements
            .iter()
            .all(|statement| match &statement.facts {
                PostgresSqlStatementKind::DoBlock { block } => block.complete,
                _ => true,
            });
    block.statements = nested.statements;
    block.diagnostics = nested.diagnostics;
    Ok(PostgresSqlStatementKind::DoBlock { block })
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

fn diagnostic(message: &str, span: &PostgresSqlSpan) -> PostgresSqlDiagnostic {
    PostgresSqlDiagnostic {
        message: message.into(),
        span: Some(span.clone()),
    }
}

pub(super) fn relocate(
    token: &mut TokenWithSpan,
    local: &Locations<'_>,
    original: &Locations<'_>,
    body: &super::body::Body<'_>,
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
