use super::evaluate::Columns;
use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::{
    sql_identifier_syntax::{literal_projection::ProjectedLiteral, Input},
    SqlObligationVerdict, SqlVerdictStatus,
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
        let application =
            crate::codebase::postgres::sql_identifier_syntax::helper_recovery::ApplicationKey::new(
                &opts.embedded,
                RULE_ID,
                index,
            );
        let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else {
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
            let mut status = SqlVerdictStatus::ProvenClear;
            let mut reasons = Vec::new();
            let mut uncertain = false;
            match &call.input {
                Input::Sql { source_span, .. } => match literals.get(&(None, *source_span)) {
                    Some(ProjectedLiteral::Direct(Ok(Some(uses)))) => {
                        for use_ in uses {
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
                                    uncertain = true;
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
                    Some(ProjectedLiteral::Direct(Err(())))
                    | Some(ProjectedLiteral::Direct(Ok(None)))
                    | None => {
                        uncertain = true;
                        status = SqlVerdictStatus::Abstain;
                        reasons.push("SQL is unanalyzable at effort low; pass a literal string or interpolation-free template directly to the imported executor or through one local const, or use effort max".into());
                        if opts.fail_unanalyzable {
                            findings.push(super::scan::unknown(&rel, call.line as usize));
                        }
                    }
                    _ => cache::mark_input_unknown(&mut status, &mut uncertain, &mut reasons),
                },
                Input::Recovered(proof) => {
                    let mut invalid = false;
                    if let Some(ProjectedLiteral::Recovered { alternatives, .. }) =
                        literals.get(&(Some(application.clone()), call.span))
                    {
                        for (sql, facts) in alternatives {
                            match facts {
                                Ok(uses) => {
                                    for use_ in uses {
                                        match columns.resolve(use_) {
                                            Ok((table, true)) => {
                                                status = SqlVerdictStatus::Violation;
                                                let mut finding = cache::recovered_finding(
                                                    root, path, sql, use_, table,
                                                );
                                                if suppressed_call {
                                                    finding.file = rel.clone();
                                                    finding.line = call.line as usize;
                                                }
                                                findings.push(finding);
                                            }
                                            Err(reason) => {
                                                uncertain = true;
                                                if status != SqlVerdictStatus::Violation {
                                                    status = SqlVerdictStatus::Abstain;
                                                }
                                                reasons.push(reason.into());
                                                if opts.fail_unanalyzable {
                                                    findings.push(super::scan::unknown(
                                                        &rel,
                                                        call.line as usize,
                                                    ));
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Err(()) => {
                                    invalid = true;
                                    uncertain = true;
                                }
                            }
                        }
                    } else {
                        invalid = true;
                    }
                    if !proof.complete() || invalid {
                        uncertain = true;
                        cache::mark_uncertain(&mut status, &mut reasons, invalid);
                        if opts.fail_unanalyzable {
                            findings.push(super::scan::unknown(&rel, call.line as usize));
                        }
                    }
                }
                _ => {
                    cache::mark_input_unknown(&mut status, &mut uncertain, &mut reasons);
                    if opts.fail_unanalyzable {
                        findings.push(super::scan::unknown(&rel, call.line as usize));
                    }
                }
            }
            if !context.wants_sql_verdicts() && !(uncertain && opts.advise_unanalyzable) {
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
                advisory: uncertain && opts.advise_unanalyzable,
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
