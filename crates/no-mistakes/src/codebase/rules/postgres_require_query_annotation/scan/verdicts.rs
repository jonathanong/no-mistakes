use super::{literal_helpers, CompiledOptions, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::{
    combine_sql_verdicts, EmbeddedSqlFileFacts, EmbeddedSqlKind, SqlEvidenceKind,
    SqlObligationVerdict, SqlVerdictStatus,
};
use std::path::Path;

mod classify;
use classify::{annotation_status, judge_sql, TextProof};

pub(super) fn record(
    file: &str,
    embedded: &EmbeddedSqlFileFacts,
    opts: &CompiledOptions,
    facts: &dyn SqlFactContext,
    invalid: &crate::fx::FxHashSet<(usize, usize)>,
    prepared: &CheckFactMap,
    path: &Path,
) {
    for (call_index, (call, span)) in embedded.calls.iter().zip(&embedded.call_spans).enumerate() {
        let (status, evidence, residual_reasons, known_prefix) =
            if let Some(result) = super::head_verdict(prepared, path, opts, *span) {
                result
            } else if !call.variants.is_empty() {
                let branches = call
                    .variants
                    .iter()
                    .enumerate()
                    .filter(|(variant_index, _)| !invalid.contains(&(call_index, *variant_index)))
                    .map(|(_, version)| annotation_status(&version.sql_text))
                    .collect::<Vec<_>>();
                if branches.is_empty() && !call.variants_truncated() {
                    (
                        SqlVerdictStatus::Abstain,
                        None,
                        vec!["invalid-composed-sql".to_string()],
                        None,
                    )
                } else if call.variants_truncated() {
                    let truncated = vec!["sql-variants-truncated".to_string()];
                    if branches.contains(&SqlVerdictStatus::Violation) {
                        (
                            SqlVerdictStatus::Violation,
                            Some(SqlEvidenceKind::VerifiedPrefix),
                            truncated,
                            composed_prefix(call),
                        )
                    } else {
                        match prepared.query_annotation_prefix(path, &opts.embedded, *span) {
                            Some(Some(prefix)) => (
                                annotation_status(prefix),
                                Some(SqlEvidenceKind::VerifiedPrefix),
                                truncated,
                                Some(prefix.clone()),
                            ),
                            Some(None) | None => (
                                SqlVerdictStatus::Abstain,
                                None,
                                vec![
                                    "sql-variants-truncated".to_string(),
                                    "unresolved-leading-sql".to_string(),
                                ],
                                composed_prefix(call),
                            ),
                        }
                    }
                } else {
                    let exhaustive = branches.len() == call.variants.len();
                    (
                        combine_sql_verdicts(&branches, exhaustive, true),
                        exhaustive.then_some(SqlEvidenceKind::ExhaustiveUnion),
                        Vec::new(),
                        composed_prefix(call),
                    )
                }
            } else if super::trusted_append::accepts(prepared, path, call, *span) {
                (
                    SqlVerdictStatus::ProvenClear,
                    Some(SqlEvidenceKind::VerifiedPrefix),
                    Vec::new(),
                    call.sql_text.clone(),
                )
            } else {
                let projected = (call.kind == EmbeddedSqlKind::Composed)
                    .then(|| prepared.query_annotation_prefix(path, &opts.embedded, *span))
                    .flatten();
                let (sql, used_projection) = match projected {
                    Some(Some(sql)) => (Some(sql.as_str()), true),
                    Some(None) => (None, true),
                    None => (call.sql_text.as_deref(), false),
                };
                let proof = if call.kind == EmbeddedSqlKind::Dynamic {
                    TextProof::Partial
                } else if used_projection {
                    TextProof::Prepared
                } else {
                    TextProof::Exact
                };
                let (status, evidence, residual) = judge_sql(sql, proof);
                (
                    status,
                    evidence,
                    residual,
                    sql.map(str::to_string).filter(|_| {
                        used_projection
                            || call.is_unanalyzable()
                            || call.kind == EmbeddedSqlKind::Composed
                    }),
                )
            };
        let invalid_composed = residual_reasons
            .iter()
            .any(|reason| reason == "invalid-composed-sql");
        let advisory = !super::loop_carried::proves_missing(prepared, path, opts, *span)
            && literal_helpers::advisory(prepared, path, opts, *span)
            && !invalid_composed
            && !call.variants_truncated();
        facts.record_sql_verdict(SqlObligationVerdict {
            profile: opts.embedded.clone(),
            rule: RULE_ID.to_string(),
            application: None,
            subcheck: "leading-annotation".to_string(),
            file: file.to_string(),
            span: *span,
            status,
            advisory,
            evidence,
            known_prefix,
            helper_lines: embedded
                .imported_call_lines
                .get(span)
                .cloned()
                .unwrap_or_default(),
            residual_reasons,
        });
    }
}

fn composed_prefix(call: &crate::codebase::postgres::EmbeddedSqlCall) -> Option<String> {
    (call.is_unanalyzable() || call.kind == EmbeddedSqlKind::Composed)
        .then(|| call.sql_text.clone())
        .flatten()
}

#[cfg(test)]
mod tests;
