use super::{annotation_status, prefix_status, EmbeddedSqlKind, SqlEvidenceKind, SqlVerdictStatus};

pub(super) fn composed_prefix(call: &crate::codebase::postgres::EmbeddedSqlCall) -> Option<String> {
    (call.is_unanalyzable() || call.kind == EmbeddedSqlKind::Composed)
        .then(|| call.sql_text.clone())
        .flatten()
}

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
