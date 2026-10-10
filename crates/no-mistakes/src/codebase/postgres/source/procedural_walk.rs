//! Classify procedural occurrences without executing SQL or evaluating conditions.
use super::body::Body;
use super::locations::Locations;
use super::types::*;
use sqlparser::{
    dialect::PostgreSqlDialect,
    tokenizer::{TokenWithSpan, Tokenizer},
};

pub(super) struct Walk {
    pub occurrences: Vec<PostgresSqlProceduralOccurrence>,
    pub legacy_stop: bool,
    pub walker_only: bool,
}

pub(super) const DYNAMIC_MESSAGE: &str = "Dynamic EXECUTE is unknown; no SQL statement is inferred";
pub(super) const UNKNOWN_MESSAGE: &str =
    "Unsupported procedural occurrence; no execution is inferred";
pub(super) const DML_MESSAGE: &str = "Static DML is a source occurrence, not an executed statement";

pub(super) fn walk(
    body: &Body<'_>,
    locations: &Locations<'_>,
    body_span: &PostgresSqlSpan,
) -> Walk {
    let Ok(tokens) = tokenize(body.sql.as_ref()) else {
        return Walk {
            occurrences: vec![unknown(body_span.clone())],
            legacy_stop: false,
            walker_only: false,
        };
    };
    let local = Locations::new(body.sql.as_ref());
    let classified = super::procedural_classify::classify(
        &tokens,
        &|start, end| map_span(&tokens, start, end, body, &local, locations, body_span),
        true,
    );
    Walk {
        occurrences: classified.occurrences,
        legacy_stop: classified.legacy_stop,
        walker_only: classified.walker_only,
    }
}

pub(super) fn finish_unparsed(block: &mut PostgresSqlProceduralBlock, span: &PostgresSqlSpan) {
    if block.occurrences.is_empty() {
        block.occurrences.push(unknown(span.clone()));
    }
    push_walk_diagnostics(block, span);
    block.complete = recognized(&block.occurrences);
}

pub(super) fn finish_parsed(block: &mut PostgresSqlProceduralBlock) {
    let safe = recognized(&block.occurrences)
        && block.diagnostics.is_empty()
        && block.statements.iter().all(|statement| {
            matches!(statement.facts, PostgresSqlStatementKind::Other)
                || super::completeness::statement(&statement.facts)
        });
    if safe {
        block.complete = true;
        return;
    }
    block
        .diagnostics
        .extend(super::procedural_occurrences::unsupported(
            &block.statements,
        ));
    block.complete = block.diagnostics.is_empty()
        && block
            .statements
            .iter()
            .all(|statement| super::completeness::statement(&statement.facts));
    // Dynamic commands stay fail-closed even when neighboring SQL facts parse.
    if has_kind(
        &block.occurrences,
        PostgresSqlProceduralOccurrenceKind::DynamicExecute,
    ) {
        block.complete = false;
    }
}

pub(super) fn unknown(span: PostgresSqlSpan) -> PostgresSqlProceduralOccurrence {
    PostgresSqlProceduralOccurrence {
        kind: PostgresSqlProceduralOccurrenceKind::Unknown,
        span,
        occurrences: Vec::new(),
    }
}

fn push_walk_diagnostics(block: &mut PostgresSqlProceduralBlock, span: &PostgresSqlSpan) {
    if has_kind(
        &block.occurrences,
        PostgresSqlProceduralOccurrenceKind::DynamicExecute,
    ) {
        block.diagnostics.push(diagnostic(DYNAMIC_MESSAGE, span));
    }
    if has_kind(
        &block.occurrences,
        PostgresSqlProceduralOccurrenceKind::Unknown,
    ) {
        block.diagnostics.push(diagnostic(UNKNOWN_MESSAGE, span));
    }
    if has_kind(&block.occurrences, PostgresSqlProceduralOccurrenceKind::Dml) {
        block.diagnostics.push(diagnostic(DML_MESSAGE, span));
    }
}

fn recognized(occurrences: &[PostgresSqlProceduralOccurrence]) -> bool {
    !occurrences.is_empty() && occurrences.iter().all(recognized_one)
}

fn recognized_one(occurrence: &PostgresSqlProceduralOccurrence) -> bool {
    matches!(
        occurrence.kind,
        PostgresSqlProceduralOccurrenceKind::Utility
            | PostgresSqlProceduralOccurrenceKind::ControlFlow
    ) && occurrence.occurrences.iter().all(recognized_one)
}

fn has_kind(
    occurrences: &[PostgresSqlProceduralOccurrence],
    kind: PostgresSqlProceduralOccurrenceKind,
) -> bool {
    occurrences
        .iter()
        .any(|occurrence| occurrence.kind == kind || has_kind(&occurrence.occurrences, kind))
}

fn diagnostic(message: &str, span: &PostgresSqlSpan) -> PostgresSqlDiagnostic {
    PostgresSqlDiagnostic {
        message: message.into(),
        span: Some(span.clone()),
    }
}

fn tokenize(sql: &str) -> Result<Vec<TokenWithSpan>, ()> {
    Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .map_err(|_| ())
}

fn map_span(
    tokens: &[TokenWithSpan],
    start: usize,
    end: usize,
    body: &Body<'_>,
    local: &Locations<'_>,
    original: &Locations<'_>,
    fallback: &PostgresSqlSpan,
) -> PostgresSqlSpan {
    let Some(start_token) = tokens.get(start.min(end)) else {
        return fallback.clone();
    };
    let Some(end_token) = tokens.get(end.max(start)) else {
        return fallback.clone();
    };
    let Some(start) = local
        .position(start_token.span.start)
        .and_then(|position| body.offset(position.offset))
    else {
        return fallback.clone();
    };
    let Some(end) = local
        .position(end_token.span.end)
        .and_then(|position| body.offset(position.offset))
    else {
        return fallback.clone();
    };
    if start > end {
        return fallback.clone();
    }
    original.range(start, end)
}
