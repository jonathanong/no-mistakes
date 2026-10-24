use super::evaluate::Columns;
use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::{
    sql_identifier_syntax::Input, SqlObligationVerdict, SqlVerdictStatus,
};
use crate::codebase::rules::PostgresFindings;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::schema::RuleDef;
use std::path::{Path, PathBuf};
mod cache;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
    index: usize,
    rule: &RuleDef,
) -> anyhow::Result<PostgresFindings> {
    let context = facts.for_sql_application_with_advice(index, rule.name.as_deref());
    let sql_files = files
        .iter()
        .filter(|file| file.extension().is_some_and(|ext| ext == "sql"))
        .cloned()
        .collect::<Vec<_>>();
    let mut findings = super::scan::scan(root, opts, &sql_files, sources, &context)?;
    let columns = Columns::new(facts.postgres_schema_catalog(&opts.schema_catalog_path)?);
    for path in files
        .iter()
        .filter(|file| crate::codebase::dependencies::extract::is_indexable(file))
    {
        let rel = relative_slash_path(root, path);
        let file = facts
            .ts
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("{RULE_ID}: prepared facts missing for {rel}"))?;
        let Some(calls) = file
            .sql_identifier_syntax
            .profiles
            .iter()
            .find(|(profile, _)| profile == &opts.embedded)
            .map(|(_, calls)| calls)
        else {
            if file.parse_error.is_some() || file.ts.parse_error.is_some() {
                continue;
            }
            anyhow::bail!("{RULE_ID}: prepared low-effort SQL syntax projection missing for {rel}");
        };
        let source = sources.read_path(path)?;
        let literals = cache::literals(file, facts, path, &source);
        for call in calls {
            let suppressed_call = crate::codebase::ts_source::matching_disable_directive(
                &source,
                Some(call.line),
                RULE_ID,
            )
            .is_some();
            let result = match &call.input {
                Input::Sql { source_span, .. } => {
                    literals.get(source_span).cloned().unwrap_or(Ok(None))
                }
                _ => Ok(None),
            };
            let mut status = SqlVerdictStatus::ProvenClear;
            let mut reasons = Vec::new();
            match result {
                Ok(Some(uses)) => {
                    for use_ in &uses {
                        match columns.resolve(use_) {
                            Ok((table, true)) => {
                                status = SqlVerdictStatus::Violation;
                                let mut finding = super::scan::finding(&rel, table, use_);
                                if suppressed_call {
                                    finding.line = call.line as usize;
                                }
                                findings.push(finding);
                            }
                            Err(reason) => {
                                if status != SqlVerdictStatus::Violation {
                                    status = SqlVerdictStatus::Abstain;
                                }
                                reasons.push(reason.into());
                                if opts.fail_unanalyzable {
                                    let line = if suppressed_call {
                                        call.line as usize
                                    } else {
                                        use_.line
                                    };
                                    findings.push(super::scan::unknown(&rel, line));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    status = SqlVerdictStatus::Abstain;
                    reasons.push("SQL is unanalyzable at effort low; pass a literal string or interpolation-free template directly to the imported executor or through one local const, or use effort max".into());
                    if opts.fail_unanalyzable {
                        findings.push(super::scan::unknown(&rel, call.line as usize));
                    }
                }
            }
            if !context.wants_sql_verdicts()
                && !(status == SqlVerdictStatus::Abstain && opts.advise_unanalyzable)
            {
                continue;
            }
            let verdict = SqlObligationVerdict {
                profile: opts.embedded.clone(),
                rule: RULE_ID.into(),
                application: None,
                subcheck: "uuid-predicates".into(),
                file: rel.clone(),
                span: call.span,
                status,
                advisory: status == SqlVerdictStatus::Abstain && opts.advise_unanalyzable,
                evidence: None,
                known_prefix: None,
                helper_lines: Vec::new(),
                residual_reasons: reasons,
            };
            if context.wants_sql_verdicts() {
                context.record_sql_verdict(verdict);
            } else {
                context.record_sql_advisory(verdict);
            }
        }
    }
    findings.sort();
    Ok(findings)
}
