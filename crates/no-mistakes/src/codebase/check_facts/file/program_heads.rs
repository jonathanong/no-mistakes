use super::super::super::CheckFactPlan;
use crate::codebase::postgres::{EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlOptions};
use crate::fx::FxHashSet;

/// Inspect already prepared call metadata before requesting a lexical index.
/// Max-only files without a named composed call pay no head visitor/index cost.
pub(super) fn candidates(
    plan: &CheckFactPlan,
    embedded: &[(EmbeddedSqlOptions, EmbeddedSqlFileFacts)],
) -> FxHashSet<(u32, u32)> {
    if !plan.query_annotation {
        return Default::default();
    }
    embedded
        .iter()
        .filter(|(profile, _)| {
            plan.query_annotation_options
                .as_ref()
                .is_none_or(|profiles| profiles.contains(profile))
        })
        .flat_map(|(_, file)| file.calls.iter().zip(&file.call_spans))
        .filter(|(call, _)| {
            call.kind == EmbeddedSqlKind::Composed
                && call.variants.is_empty()
                && call
                    .sql_text
                    .as_deref()
                    .and_then(crate::codebase::postgres::leading_query_annotation)
                    .is_some()
        })
        .map(|(_, span)| *span)
        .collect()
}

pub(super) fn has_annotation_executors(
    plan: &CheckFactPlan,
    embedded: &[(EmbeddedSqlOptions, EmbeddedSqlFileFacts)],
) -> bool {
    embedded.iter().any(|(profile, file)| {
        !file.call_spans.is_empty()
            && plan
                .query_annotation_options
                .as_ref()
                .is_none_or(|profiles| profiles.contains(profile))
    })
}
