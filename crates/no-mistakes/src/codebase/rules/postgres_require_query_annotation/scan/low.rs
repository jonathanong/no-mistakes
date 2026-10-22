use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{SqlEvidenceKind, SqlObligationVerdict, SqlVerdictStatus};
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
            let visible = heads.is_some_and(|heads| heads.accepts(*span));
            if facts.wants_sql_verdicts() {
                facts.record_sql_verdict(SqlObligationVerdict {
                    profile: opts.embedded.clone(),
                    rule: RULE_ID.to_string(),
                    application: None,
                    subcheck: "leading-annotation".to_string(),
                    file: rel.clone(),
                    span: *span,
                    status: if visible {
                        SqlVerdictStatus::ProvenClear
                    } else {
                        SqlVerdictStatus::Abstain
                    },
                    advisory: opts.unanalyzable_sql == UnanalyzableSql::Advise,
                    evidence: visible.then_some(SqlEvidenceKind::VerifiedPrefix),
                    known_prefix: None,
                    helper_lines: Vec::new(),
                    residual_reasons: if visible {
                        Vec::new()
                    } else {
                        vec![format!(
                            "leading annotation is not visible at effort low; {REWRITE}"
                        )]
                    },
                });
            }
            if !visible && opts.unanalyzable_sql == UnanalyzableSql::Report {
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
