use crate::codebase::postgres::{
    leading_query_annotation, sql_requires_query_annotation, SqlEvidenceKind, SqlVerdictStatus,
};

pub(super) enum TextProof {
    Exact,
    Prepared,
    Partial,
}

pub(super) fn judge_sql(
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

pub(super) fn annotation_status(sql: &str) -> SqlVerdictStatus {
    if sql_requires_query_annotation(sql) {
        SqlVerdictStatus::Violation
    } else {
        SqlVerdictStatus::ProvenClear
    }
}
