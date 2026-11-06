use super::super::Span;
use crate::codebase::{
    check_facts::{
        sql_source_prerequisites::{FailedSqlSources, SqlProjection, SqlSourceKey},
        CheckFileFacts,
    },
    ts_source::FileIdMap,
};
use std::path::Path;

pub(super) fn record(
    files: &mut FileIdMap<CheckFileFacts>,
    owner: &Path,
    span: Span,
    failures: FailedSqlSources,
) {
    if failures.is_empty() {
        return;
    }
    let Some(file) = files.get_mut(owner) else {
        return;
    };
    let mut spans = std::collections::BTreeSet::from([span]);
    if let Some(heads) = &file.query_annotation_heads {
        for condition in &heads.conditional_effects {
            if condition.required.contains(&span) {
                spans.extend(condition.spans.iter().copied());
            }
        }
    }
    for (profile, embedded) in &file.embedded_sql {
        for span in spans.iter().copied().filter(|span| {
            embedded.call_spans.contains(span)
                || embedded
                    .pending_relative
                    .call_spans
                    .values()
                    .any(|value| value == span)
        }) {
            file.sql_source_prerequisites.record(
                &SqlSourceKey::profile(SqlProjection::AnnotationHeads, profile),
                span,
                failures.iter().cloned(),
            );
        }
    }
}

#[cfg(test)]
mod tests;
