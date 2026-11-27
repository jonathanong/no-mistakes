use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::{SqlDuplicateKeyedLookupFact, SqlFactSite};
use crate::codebase::postgres::{EmbeddedSqlCall, SqlBranchJudgment, SqlPolicyVerdictContext};
use crate::codebase::rules::{PostgresFindings, RuleFinding, VariantFindingDedup};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    facts: &dyn SqlFactContext,
    sources: &SourceStore,
) -> Result<PostgresFindings> {
    let prepared = facts.prepared();
    let mut findings = PostgresFindings::default();
    let mut dedup = VariantFindingDedup::for_root(root, sources);
    for path in files {
        let rel = relative_slash_path(root, path);
        let profile = (!rel.ends_with(".sql")).then_some(&opts.embedded);
        let statements = match prepared.postgres_statements(path, profile) {
            Ok(statements) => statements,
            Err(_) if rel.ends_with(".sql") && prepared.postgres().sql_source_not_found(path) => {
                continue
            }
            Err(error) => return Err(error),
        };
        let embedded = profile
            .map(|profile| prepared.embedded_sql(path, profile))
            .transpose()?;
        if let Some(embedded) = embedded.filter(|_| facts.wants_sql_verdicts() || opts.advise_unanalyzable) {
            crate::codebase::postgres::record_sql_policy_verdicts(
                SqlPolicyVerdictContext {
                    facts,
                    embedded,
                    statements,
                },
                RULE_ID,
                &rel,
                &opts.embedded,
                &["duplicate-keyed-lookups".to_owned()],
                opts.advise_unanalyzable,
                |_, _, statement| {
                    if !statement.duplicate_keyed_lookups.is_empty() {
                        SqlBranchJudgment::violation(true)
                    } else {
                        SqlBranchJudgment::clear()
                    }
                },
            );
        }
        if opts.fail_unanalyzable {
            if let Some(file) = embedded {
                for call in &file.calls {
                    if call.is_unanalyzable() {
                        findings.push(unanalyzable_call(&rel, call));
                    }
                }
            }
        }
        let source = crate::codebase::rules::read_language_source(sources, path);
        for statement in statements {
            if opts.advise_unanalyzable && profile.is_none() {
                crate::codebase::rules::native_sql_advice::record_parse_failures(
                    facts,
                    RULE_ID,
                    &opts.embedded,
                    &rel,
                    source.as_deref().unwrap_or_default(),
                    statement,
                );
            }
            if opts.fail_unanalyzable {
                for (site, line) in statement.parse_failure_sites() {
                    dedup.push(
                        &mut findings,
                        statement,
                        site,
                        source.as_deref(),
                        unanalyzable_line(&rel, line),
                    );
                }
            }
            for (index, lookup) in statement.duplicate_keyed_lookups.iter().enumerate() {
                dedup.push(
                    &mut findings,
                    statement,
                    SqlFactSite::DuplicateKeyedLookup(index),
                    source.as_deref(),
                    finding(&rel, lookup),
                );
            }
        }
    }
    findings.sort();
    Ok(findings)
}

fn finding(file: &str, lookup: &SqlDuplicateKeyedLookupFact) -> RuleFinding {
    let line = lookup.line.max(1);
    let first_line = lookup.first_line.max(1);
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_owned(),
        file: file.to_owned(),
        line,
        message: format!(
            "{file}:{line}: duplicate keyed lookup of `{}` on `{}` (already looked up at line {first_line}). Verify that consolidation preserves row multiplicity and unmatched rows.",
            lookup.relation, lookup.key_display
        ),
        import: None,
        target: Some(lookup.relation.clone()),
    }
}

fn unanalyzable_call(file: &str, call: &EmbeddedSqlCall) -> RuleFinding {
    unanalyzable_line(file, call.line.max(1) as usize)
}

fn unanalyzable_line(file: &str, line: usize) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_owned(),
        file: file.to_owned(),
        line,
        message: format!(
            "{file}:{line}: executed SQL is not statically recoverable, so duplicate keyed lookups cannot be checked; pass static SQL or set unanalyzable: advise or ignore"
        ),
        import: None,
        target: Some("unanalyzable".to_owned()),
    }
}
