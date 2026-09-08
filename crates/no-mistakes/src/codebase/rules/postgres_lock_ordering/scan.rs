use super::directive::{contains_for_update, has_safe_directive};
use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, extract_locking_select_metadata, PostgresSchemaOptions, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(super) const UNPARSEABLE_TARGET: &str = "unparseable";
pub(super) const LOCK_ORDERING_TARGET: &str = "lock-ordering";

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let catalog = opts
        .schema_catalog_path
        .as_deref()
        .map(|path| SchemaCatalog::load(root, path, sources))
        .transpose()?;
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &PostgresSchemaOptions::default(),
        &opts.embedded,
    )
    .with_context(|| format!("{RULE_ID} failed to collect embedded SQL facts"))?;
    let mut findings = Vec::new();
    for file in facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path).unwrap_or_default();
        for call in &file.calls {
            findings.extend(catalog.as_ref().map_or_else(
                || findings_for_call(&rel, &source, call, opts),
                |catalog| findings_for_call_with_catalog(&rel, &source, call, opts, Some(catalog)),
            ));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

pub(super) fn findings_for_call(
    file: &str,
    source: &str,
    call: &crate::codebase::postgres::EmbeddedSqlCall,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    findings_for_call_with_catalog(file, source, call, opts, None)
}

fn findings_for_call_with_catalog(
    file: &str,
    source: &str,
    call: &crate::codebase::postgres::EmbeddedSqlCall,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
) -> Vec<RuleFinding> {
    let Some(sql) = call.sql_text.as_deref() else {
        return Vec::new();
    };
    if !contains_for_update(sql) {
        return Vec::new();
    }
    if has_safe_directive(source, call.line, sql, &opts.safe_directive) {
        return Vec::new();
    }
    match extract_locking_select_metadata(sql) {
        Err(_) => vec![finding(
            file,
            call.line,
            unparseable_message(file, call.line, &opts.safe_directive),
            UNPARSEABLE_TARGET,
        )],
        Ok(locks) => {
            if locks.iter().any(|lock| {
                lock.has_multi_row_predicate && !lock.has_order_by && !lock.skips_locked_rows
            }) {
                vec![finding(
                    file,
                    call.line,
                    lock_ordering_message(file, call.line, &opts.safe_directive),
                    LOCK_ORDERING_TARGET,
                )]
            } else if let Some(catalog) = catalog {
                if locks.iter().any(|lock| {
                    lock.has_multi_row_predicate
                        && !lock.skips_locked_rows
                        && !lock
                            .table
                            .as_deref()
                            .zip(lock.order.as_deref())
                            .is_some_and(|(table, order)| {
                                catalog.has_canonical_prefix(table, order)
                            })
                }) {
                    vec![finding(
                        file,
                        call.line,
                        canonical_order_message(file, call.line, &opts.safe_directive),
                        LOCK_ORDERING_TARGET,
                    )]
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
    }
}

fn canonical_order_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: multi-row FOR UPDATE ORDER BY must begin with a valid schema-catalog unique-key order; add the catalog key prefix, use SKIP LOCKED, or add a `{directive}` comment"
    )
}

fn finding(file: &str, line: u32, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: line as usize,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}

fn lock_ordering_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: multi-row FOR UPDATE without ORDER BY or SKIP LOCKED can deadlock (ABBA); add ORDER BY, use SKIP LOCKED, or add a `{directive}` comment"
    )
}

fn unparseable_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: keep FOR UPDATE SQL parseable so lock ordering can be checked, or add a `{directive}` comment"
    )
}
