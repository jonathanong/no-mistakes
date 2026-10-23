use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    EmbeddedSqlKind, SqlEvidenceKind, SqlObligationVerdict, SqlVerdictStatus,
};
use crate::codebase::rules::{
    postgres_require_query_annotation::UnanalyzableSql, PostgresFindings,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const REWRITE: &str = "use a leading literal at the executor call, query(\"/* name */ \" + sql), or one local const initialized with that leading literal and passed directly: const statement = \"/* name */ \" + sql; query(statement); any tagged template with that literal head also qualifies in either position";

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    facts: &dyn SqlFactContext,
    observer: Option<&crate::diagnostics::InvocationObserver>,
) -> Result<PostgresFindings> {
    let prepared = facts.prepared();
    let mut findings = PostgresFindings::default();
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let file = prepared
            .embedded_sql(path, &opts.embedded)
            .with_context(|| format!("{RULE_ID} failed to collect embedded SQL facts"))?;
        let rel = relative_slash_path(root, &file.path);
        let heads = prepared
            .ts
            .get(path)
            .and_then(|file| file.query_annotation_heads.as_deref());
        if let Some(observer) = observer {
            observer.increment("postgres.annotation.low.calls", file.calls.len() as u64);
        }
        for (call, span) in file.calls.iter().zip(&file.call_spans) {
            let helper =
                heads.and_then(|heads| heads.helper(*span, &opts.embedded.sql_literal_helpers));
            use crate::codebase::postgres::query_annotation::heads::HeadProof;
            let visible = helper.map_or_else(
                || heads.is_some_and(|heads| heads.accepts(*span)),
                |proof| proof == HeadProof::Literal,
            );
            let definite_violation = helper.map_or_else(
                || heads.is_some_and(|heads| heads.proves_missing(*span)),
                |proof| proof == HeadProof::Missing,
            );
            let status = if visible {
                SqlVerdictStatus::ProvenClear
            } else if definite_violation {
                SqlVerdictStatus::Violation
            } else {
                SqlVerdictStatus::Abstain
            };
            if facts.wants_sql_verdicts() {
                facts.record_sql_verdict(SqlObligationVerdict {
                    profile: opts.embedded.clone(),
                    rule: RULE_ID.to_string(),
                    application: None,
                    subcheck: "leading-annotation".to_string(),
                    file: rel.clone(),
                    span: *span,
                    status,
                    advisory: status == SqlVerdictStatus::Abstain
                        && opts.unanalyzable_sql == UnanalyzableSql::Advise,
                    evidence: match status {
                        SqlVerdictStatus::ProvenClear => Some(SqlEvidenceKind::VerifiedPrefix),
                        SqlVerdictStatus::Violation if call.kind == EmbeddedSqlKind::Dynamic => {
                            Some(SqlEvidenceKind::VerifiedPrefix)
                        }
                        SqlVerdictStatus::Violation => Some(SqlEvidenceKind::Exact),
                        SqlVerdictStatus::Abstain => None,
                    },
                    known_prefix: None,
                    helper_lines: Vec::new(),
                    residual_reasons: if status == SqlVerdictStatus::Abstain {
                        vec![format!(
                            "leading annotation is not visible at effort low; {REWRITE}"
                        )]
                    } else {
                        Vec::new()
                    },
                });
            }
            if definite_violation {
                findings.push_at(
                    RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: call.line as usize,
                        message: format!(
                            "{rel}:{}: query execution must start with a /* name */ annotation",
                            call.line
                        ),
                        import: None,
                        target: Some("annotation".to_string()),
                    },
                    Some(span.0 as usize),
                );
            } else if !visible && opts.unanalyzable_sql == UnanalyzableSql::Report {
                findings.push_at(
                    RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: call.line as usize,
                        message: format!(
                            "{rel}:{}: query annotation is unanalyzable at effort low; {REWRITE}",
                            call.line
                        ),
                        import: None,
                        target: Some("annotation".to_string()),
                    },
                    Some(span.0 as usize),
                );
            }
        }
    }
    findings.sort();
    Ok(findings)
}

#[cfg(test)]
mod tests;
