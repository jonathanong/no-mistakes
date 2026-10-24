use super::super::LocatedStatement;
use super::queries;
mod unsupported;
use crate::codebase::postgres::types::SqlUnanalyzableStatement;
use sqlparser::ast::{visit_statements, DiscardObject, Statement};
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::ops::ControlFlow;
use std::sync::Arc;

pub(super) fn guarded_action(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
) -> Option<LocatedStatement> {
    let start = queries::guarded_action_start(tokens)?;
    let suffix = original?.get(start..)?;
    let mut marker = LocatedStatement::functions(
        Statement::Discard {
            object_type: DiscardObject::ALL,
        },
        false,
    );
    marker.recovery_failure = unrecoverable_chunk(suffix, 0);
    marker.recovery_failure.as_mut()?.source_order.clear();
    Some(marker)
}

pub(super) fn unrecoverable_chunk(
    tokens: &[TokenWithSpan],
    ordinal: usize,
) -> Option<SqlUnanalyzableStatement> {
    let (location, head) = super::super::super::statement_head::head(tokens)?;
    let line = location.line.max(1) as usize;
    let column = location.column.max(1) as usize;
    let sql = tokens
        .iter()
        .map(|token| token.token.to_string())
        .collect::<String>();
    Some(SqlUnanalyzableStatement {
        head,
        valid_unsupported: unsupported::valid(tokens),
        ..failed_source(&sql, line, column, ordinal)
    })
}

pub(in super::super) fn failed_source(
    sql: &str,
    line: usize,
    column: usize,
    ordinal: usize,
) -> SqlUnanalyzableStatement {
    let masked = crate::codebase::postgres::statements::mask_quoted_sql(sql);
    let start = sql
        .char_indices()
        .zip(masked.chars())
        .find_map(|((offset, _), character)| (!character.is_whitespace()).then_some(offset))
        .unwrap_or(0);
    SqlUnanalyzableStatement {
        line,
        column,
        head: sql[start..]
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" "),
        valid_unsupported: false,
        source_order: vec![ordinal],
        insert_keyword_count: crate::codebase::postgres::statements::insert_keyword_count(&masked),
        recovered_insert_count: 0,
        has_top_level_not_exists:
            crate::codebase::postgres::statements::has_top_level_not_exists_in(sql),
        schema_unanalyzable: true,
        statement_unanalyzable: true,
    }
}

/// Schema uncertainty belongs to the routine wrapper when its body cannot be lexed.
/// Statement consumers retain the exact lexical child captured by fallback instead.
pub(super) fn marker(
    tokens: &Arc<[TokenWithSpan]>,
    ordinal: usize,
    lexical_wrapper: bool,
    produced: &[LocatedStatement],
) -> LocatedStatement {
    let mut marker = LocatedStatement::functions(
        Statement::Discard {
            object_type: DiscardObject::ALL,
        },
        false,
    );
    marker.recovery_failure = unrecoverable_chunk(tokens, ordinal);
    if lexical_wrapper
        && produced
            .iter()
            .any(|item| item.chunk_tokens.is_none() && item.recovery_failure.is_some())
    {
        if let Some(failure) = &mut marker.recovery_failure {
            failure.statement_unanalyzable = false;
        }
    }
    marker.chunk_tokens = Some(Arc::clone(tokens));
    marker
}

/// Capture partial recovery before an enclosing body replaces the leaf chunk tokens.
pub(super) fn partial_chunk(tokens: Option<&[TokenWithSpan]>, produced: &mut [LocatedStatement]) {
    if produced.iter().all(|item| item.complete)
        || produced.iter().any(|item| item.recovery_failure.is_some())
    {
        return;
    }
    let Some(mut failure) = tokens.and_then(|tokens| unrecoverable_chunk(tokens, 0)) else {
        return;
    };
    failure.source_order.clear();
    failure.schema_unanalyzable = false;
    for item in produced.iter().filter(|item| !item.function_projection) {
        let mut executed = Vec::new();
        crate::codebase::postgres::statements::walk_executed(&item.statement, &mut executed);
        for statement in executed {
            if matches!(statement, Statement::Insert(_) | Statement::Query(_)) {
                let _: ControlFlow<()> = visit_statements(statement, |statement| {
                    failure.recovered_insert_count +=
                        usize::from(matches!(statement, Statement::Insert(_)));
                    ControlFlow::Continue(())
                });
            }
        }
    }
    if let Some(first) = produced.first_mut() {
        first.recovery_failure = Some(failure);
    }
}

#[cfg(test)]
mod tests;
