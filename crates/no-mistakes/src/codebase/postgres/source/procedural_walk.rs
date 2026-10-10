//! Classify procedural occurrences without executing SQL or evaluating conditions.
use super::body::Body;
use super::locations::Locations;
use super::types::*;
use sqlparser::tokenizer::TokenWithSpan;

pub(super) struct Walk {
    pub occurrences: Vec<PostgresSqlProceduralOccurrence>,
    pub legacy_stop: bool,
    pub walker_only: bool,
}

pub(super) const DYNAMIC_MESSAGE: &str = "Dynamic EXECUTE is unknown; no SQL statement is inferred";
pub(super) const UNKNOWN_MESSAGE: &str =
    "Unsupported procedural occurrence; no execution is inferred";
pub(super) const DML_MESSAGE: &str = "Static DML is a source occurrence, not an executed statement";

pub(super) fn walk(tokens: &[TokenWithSpan], body: &Body<'_>, locations: &Locations<'_>) -> Walk {
    let local = Locations::new(body.sql.as_ref());
    let classified = super::procedural_classify::classify(
        tokens,
        &|start, end| map_span(tokens, start, end, body, &local, locations),
        true,
    );
    Walk {
        occurrences: classified.occurrences,
        legacy_stop: classified.legacy_stop,
        walker_only: classified.walker_only,
    }
}

pub(super) fn finish_unparsed(block: &mut PostgresSqlProceduralBlock, span: &PostgresSqlSpan) {
    push_walk_diagnostics(block, span);
    block.complete = recognized(&block.occurrences);
}

pub(super) fn finish_parsed(block: &mut PostgresSqlProceduralBlock, retain_walker: bool) {
    let safe = recognized(&block.occurrences)
        && block.diagnostics.is_empty()
        && block.statements.iter().all(|statement| {
            matches!(statement.facts, PostgresSqlStatementKind::Other)
                || super::completeness::statement(&statement.facts)
        });
    if safe {
        block.complete = true;
    } else {
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
    // Omitted walker syntax is not executed. A parsed sibling must not hide it.
    if retain_walker {
        note_omitted(block);
    }
}

fn note_omitted(block: &mut PostgresSqlProceduralBlock) {
    let omitted = block
        .occurrences
        .iter()
        .any(|occurrence| !sql_kind(occurrence.kind) && !recognized_one(occurrence));
    if !omitted {
        return;
    }
    if omitted_has(
        &block.occurrences,
        PostgresSqlProceduralOccurrenceKind::DynamicExecute,
    ) {
        block
            .diagnostics
            .push(diagnostic(DYNAMIC_MESSAGE, &block.body_span));
    }
    if omitted_has(
        &block.occurrences,
        PostgresSqlProceduralOccurrenceKind::Unknown,
    ) {
        block
            .diagnostics
            .push(diagnostic(UNKNOWN_MESSAGE, &block.body_span));
    }
    if omitted_has(&block.occurrences, PostgresSqlProceduralOccurrenceKind::Dml) {
        block
            .diagnostics
            .push(diagnostic(DML_MESSAGE, &block.body_span));
    }
    block.complete = false;
}

fn sql_kind(kind: PostgresSqlProceduralOccurrenceKind) -> bool {
    matches!(
        kind,
        PostgresSqlProceduralOccurrenceKind::Utility | PostgresSqlProceduralOccurrenceKind::Dml
    )
}

fn omitted_has(
    occurrences: &[PostgresSqlProceduralOccurrence],
    kind: PostgresSqlProceduralOccurrenceKind,
) -> bool {
    occurrences.iter().any(|occurrence| {
        !sql_kind(occurrence.kind)
            && (occurrence.kind == kind || has_kind(&occurrence.occurrences, kind))
    })
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

fn map_span(
    tokens: &[TokenWithSpan],
    start: usize,
    end: usize,
    body: &Body<'_>,
    local: &Locations<'_>,
    original: &Locations<'_>,
) -> PostgresSqlSpan {
    // Indexes address the prepared body tokens, not a second lexer inventory.
    let start_token = &tokens[start.min(end)];
    let end_token = &tokens[end.max(start)];
    let start = local
        .position(start_token.span.start)
        .and_then(|position| body.offset(position.offset))
        .expect("procedural occurrence start");
    let end = local
        .position(end_token.span.end)
        .and_then(|position| body.offset(position.offset))
        .expect("procedural occurrence end");
    original.range(start, end)
}
