use super::CompiledOptions;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use std::path::Path;

pub(super) fn proof(
    facts: &CheckFactMap,
    path: &Path,
    opts: &CompiledOptions,
    span: (u32, u32),
) -> Option<HeadProof> {
    if opts.embedded.sql_literal_helpers.is_empty() {
        return None;
    }
    facts
        .ts
        .get(path)?
        .query_annotation_heads
        .as_deref()?
        .helper(span, &opts.embedded.sql_literal_helpers)
}

type HelperVerdict = (
    crate::codebase::postgres::SqlVerdictStatus,
    Option<crate::codebase::postgres::SqlEvidenceKind>,
    Vec<String>,
    Option<String>,
);

pub(super) fn verdict(
    facts: &CheckFactMap,
    path: &Path,
    opts: &CompiledOptions,
    span: (u32, u32),
) -> Option<HelperVerdict> {
    use crate::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};
    let status = match proof(facts, path, opts, span)? {
        HeadProof::Literal => SqlVerdictStatus::ProvenClear,
        HeadProof::Missing => SqlVerdictStatus::Violation,
        HeadProof::Unproven => SqlVerdictStatus::Abstain,
    };
    Some((
        status,
        (status != SqlVerdictStatus::Abstain).then_some(SqlEvidenceKind::Exact),
        if status == SqlVerdictStatus::Abstain {
            vec!["nonliteral-sql-helper-argument".to_string()]
        } else {
            Vec::new()
        },
        None,
    ))
}

pub(super) fn advisory(
    facts: &CheckFactMap,
    path: &Path,
    opts: &CompiledOptions,
    span: (u32, u32),
) -> bool {
    opts.unanalyzable_sql == super::super::UnanalyzableSql::Advise
        && proof(facts, path, opts, span).is_none_or(|proof| proof == HeadProof::Unproven)
}
