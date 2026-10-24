use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::{
    sql_identifier_syntax::{Input, SqlCall},
    SqlObligationVerdict, SqlVerdictStatus,
};
use crate::codebase::rules::{PostgresFindings, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::schema::RuleDef;
use anyhow::Result;
use std::path::{Path, PathBuf};

mod cache;

const REWRITE: &str = "pass valid SQL as a literal string or interpolation-free template directly to the imported executor or through one local const; remove JavaScript escapes or use effort max for inferred SQL";

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    facts: &CheckFactMap,
    index: usize,
    rule: &RuleDef,
    sources: &SourceStore,
) -> Result<PostgresFindings> {
    let application = facts.for_sql_application_with_advice(index, rule.name.as_deref());
    let context: &dyn SqlFactContext = &application;
    let sql_files = files
        .iter()
        .filter(|file| file.extension().is_some_and(|extension| extension == "sql"))
        .cloned()
        .collect::<Vec<_>>();
    let mut findings = super::scan::scan(root, opts, &sql_files, context, sources)?;
    let observer = crate::diagnostics::current();
    for path in files
        .iter()
        .filter(|file| !file.extension().is_some_and(|extension| extension == "sql"))
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
        let calls = file.sql_identifier_syntax.calls_for(&application);
        let Some(calls) = calls else {
            if file.parse_error.is_some() || file.ts.parse_error.is_some() {
                continue;
            }
            anyhow::bail!("{RULE_ID}: prepared low-effort SQL syntax projection missing for {rel}");
        };
        let source = sources.read_path(path)?;
        let mut ordinal = 0;
        let literals = cache::literals(file, facts, path, &source);
        for call in calls {
            if let Some(observer) = &observer {
                observer.increment("postgres.no_offset.low.calls", 1);
            }
            let is_unknown;
            let mut invalid = false;
            let mut has_violation = false;
            match &call.input {
                Input::Sql { source_span, .. } => {
                    let result = literals
                        .get(&(None, *source_span))
                        .map(|value| match value {
                            crate::codebase::postgres::sql_identifier_syntax::literal_projection::ProjectedLiteral::Direct(result) => result.clone(),
                            _ => Ok(None),
                        })
                        .unwrap_or(Ok(None));
                    is_unknown = !matches!(&result, Ok(Some(_)));
                    invalid = result.is_err();
                    if let Ok(Some(offsets)) = result {
                        has_violation = !offsets.is_empty();
                        for mut offset in offsets {
                            ordinal += 1;
                            if crate::codebase::ts_source::matching_disable_directive(
                                &source,
                                Some(call.line),
                                RULE_ID,
                            )
                            .is_some()
                            {
                                offset.line = call.line as usize;
                            }
                            findings.push(super::scan::finding(&rel, &offset, ordinal));
                        }
                    }
                }
                Input::Recovered(proof) => {
                    match literals.get(&(Some(application.clone()), call.span)) {
                        Some(crate::codebase::postgres::sql_identifier_syntax::literal_projection::ProjectedLiteral::Recovered { alternatives, .. }) => {
                            for (sql, facts) in alternatives {
                                match facts {
                                    Ok(offsets) => {
                                        has_violation |= !offsets.is_empty();
                                        for offset in offsets {
                                            ordinal += 1;
                                            let (origin, line) = crate::codebase::postgres::sql_identifier_syntax::helper_recovery::Recovered::position(sql, offset.line, offset.column, path);
                                            let mut offset = *offset;
                                            offset.line = line;
                                            let mut origin_rel = relative_slash_path(root, origin);
                                            if crate::codebase::ts_source::matching_disable_directive(&source, Some(call.line), RULE_ID).is_some() {
                                                origin_rel = rel.clone();
                                                offset.line = call.line as usize;
                                            }
                                            findings.push(super::scan::finding(&origin_rel, &offset, ordinal));
                                        }
                                    }
                                    Err(()) => invalid = true,
                                }
                            }
                        },
                        _ => invalid = true,
                    }
                    is_unknown = !proof.complete() || invalid;
                }
                _ => is_unknown = true,
            }
            if is_unknown && opts.fail_unanalyzable {
                findings.push(unknown(&rel, call));
            }
            if !context.wants_sql_verdicts() && !(is_unknown && opts.advise_unanalyzable) {
                continue;
            }
            let verdict = SqlObligationVerdict {
                profile: opts.embedded.clone(),
                rule: RULE_ID.into(),
                application: None,
                subcheck: "offset".into(),
                file: rel.clone(),
                span: call.span,
                status: if has_violation {
                    SqlVerdictStatus::Violation
                } else if is_unknown {
                    SqlVerdictStatus::Abstain
                } else {
                    SqlVerdictStatus::ProvenClear
                },
                advisory: is_unknown && opts.advise_unanalyzable,
                evidence: None,
                known_prefix: None,
                helper_lines: Vec::new(),
                residual_reasons: if invalid {
                    vec!["invalid-sql".into()]
                } else if is_unknown {
                    vec![format!("SQL is unanalyzable at effort low; {REWRITE}")]
                } else {
                    Vec::new()
                },
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

fn unknown(file: &str, call: &SqlCall) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line: call.line as usize,
        message: format!(
            "{file}:{}: SQL is unanalyzable at effort low, so an OFFSET cannot be ruled out; {REWRITE}",
            call.line
        ),
        import: None,
        target: Some("unanalyzable".into()),
    }
}
