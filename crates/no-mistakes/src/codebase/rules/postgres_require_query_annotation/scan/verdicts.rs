use super::RULE_ID;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    combine_sql_verdicts, leading_query_annotation, sql_requires_query_annotation,
    EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlOptions, SqlEvidenceKind,
    SqlObligationVerdict, SqlVerdictStatus,
};

pub(super) fn record(
    file: &str,
    embedded: &EmbeddedSqlFileFacts,
    profile: &EmbeddedSqlOptions,
    advisory: bool,
    facts: &dyn SqlFactContext,
    invalid: &crate::fx::FxHashSet<(usize, usize)>,
) {
    for (call_index, (call, span)) in embedded.calls.iter().zip(&embedded.call_spans).enumerate() {
        let (status, evidence, residual_reasons) = if !call.variants.is_empty() {
            let branches = call
                .variants
                .iter()
                .enumerate()
                .filter(|(variant_index, _)| !invalid.contains(&(call_index, *variant_index)))
                .map(|(_, version)| annotation_status(&version.sql_text))
                .collect::<Vec<_>>();
            let exhaustive = !call.variants_truncated() && branches.len() == call.variants.len();
            (
                combine_sql_verdicts(&branches, exhaustive, true),
                exhaustive.then_some(SqlEvidenceKind::ExhaustiveUnion),
                Vec::new(),
            )
        } else {
            match (call.kind, call.sql_text.as_deref()) {
                (EmbeddedSqlKind::Dynamic, Some(prefix)) => {
                    let status = prefix_status(prefix);
                    // A complete leading annotation cannot be changed by an opaque tail.
                    let residual = (status != SqlVerdictStatus::ProvenClear)
                        .then(|| "opaque-sql-tail".to_string())
                        .into_iter()
                        .collect();
                    (
                        status,
                        (status != SqlVerdictStatus::Abstain)
                            .then_some(SqlEvidenceKind::VerifiedPrefix),
                        residual,
                    )
                }
                (_, Some(sql)) => (
                    annotation_status(sql),
                    Some(SqlEvidenceKind::Exact),
                    Vec::new(),
                ),
                (_, None) => (
                    SqlVerdictStatus::Abstain,
                    None,
                    vec!["unresolved-leading-sql".to_string()],
                ),
            }
        };
        facts.record_sql_verdict(SqlObligationVerdict {
            profile: profile.clone(),
            rule: RULE_ID.to_string(),
            application: None,
            subcheck: "leading-annotation".to_string(),
            file: file.to_string(),
            span: *span,
            status,
            advisory,
            evidence,
            known_prefix: (call.is_unanalyzable() || call.kind == EmbeddedSqlKind::Composed)
                .then(|| call.sql_text.clone())
                .flatten(),
            helper_lines: embedded
                .imported_call_lines
                .get(span)
                .cloned()
                .unwrap_or_default(),
            residual_reasons,
        });
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
