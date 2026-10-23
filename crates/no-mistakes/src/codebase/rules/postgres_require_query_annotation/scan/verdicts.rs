use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::{
    combine_sql_verdicts, leading_query_annotation, sql_requires_query_annotation,
    EmbeddedSqlFileFacts, EmbeddedSqlKind, SqlEvidenceKind, SqlObligationVerdict, SqlVerdictStatus,
};
use crate::codebase::rules::postgres_require_query_annotation::UnanalyzableSql;
use std::path::Path;

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
        let (status, evidence, residual_reasons, known_prefix) = if !call.variants.is_empty() {
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
        let advisory = opts.unanalyzable_sql == UnanalyzableSql::Advise
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

enum TextProof {
    Exact,
    Prepared,
    Partial,
}

fn judge_sql(
    sql: Option<&str>,
    proof: TextProof,
) -> (SqlVerdictStatus, Option<SqlEvidenceKind>, Vec<String>) {
    match sql {
        Some(sql) => {
            let status = if matches!(proof, TextProof::Partial) {
                prefix_status(sql)
            } else {
                annotation_status(sql)
            };
            let verified_prefix = !matches!(proof, TextProof::Exact);
            let evidence = (status != SqlVerdictStatus::Abstain).then_some(if verified_prefix {
                SqlEvidenceKind::VerifiedPrefix
            } else {
                SqlEvidenceKind::Exact
            });
            let residual = (status != SqlVerdictStatus::ProvenClear && verified_prefix)
                .then(|| "opaque-sql-tail".to_string())
                .into_iter()
                .collect();
            (status, evidence, residual)
        }
        None => (
            SqlVerdictStatus::Abstain,
            None,
            vec!["unresolved-leading-sql".to_string()],
        ),
    }
}

/// A known prefix can prove a leading annotation or rule out both an
/// annotation and a transaction command. An incomplete comment or command
/// name keeps the implicit unknown branch alive.
pub(super) fn prefix_status(prefix: &str) -> SqlVerdictStatus {
    if leading_query_annotation(prefix).is_some() {
        return SqlVerdictStatus::ProvenClear;
    }
    let leading = prefix.trim_start();
    if leading.is_empty() || leading.starts_with('/') {
        return SqlVerdictStatus::Abstain;
    }
    let upper = leading.to_ascii_uppercase();
    if ["BEGIN", "COMMIT", "ROLLBACK"]
        .iter()
        .any(|command| command.starts_with(&upper) || upper.starts_with(command))
    {
        return SqlVerdictStatus::Abstain;
    }
    SqlVerdictStatus::Violation
}

fn annotation_status(sql: &str) -> SqlVerdictStatus {
    if sql_requires_query_annotation(sql) {
        SqlVerdictStatus::Violation
    } else {
        SqlVerdictStatus::ProvenClear
    }
}

#[cfg(test)]
mod tests;
