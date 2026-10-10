use super::catalog_check::{locks_interpolated_relation, locks_single_row, orders_by_catalog_key};
use super::directive::{contains_for_update, has_safe_directive};
use super::{CompiledOptions, RULE_ID};
use crate::codebase::postgres::{
    extract_locking_select_metadata_with_placeholders, EmbeddedSqlKind, LockingSelectMetadata,
    SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(super) const UNPARSEABLE_TARGET: &str = "unparseable";
pub(super) const UNRESOLVED_RELATION_TARGET: &str = "unresolved-relation";
pub(super) const LOCK_ORDERING_TARGET: &str = "lock-ordering";
pub(super) const UNANALYZABLE_TARGET: &str = "unanalyzable";

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let catalog = opts
        .schema_catalog_path
        .as_deref()
        .map(|path| facts.postgres_ordering_catalog(path))
        .transpose()?;
    let mut findings = Vec::new();
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let file = facts.embedded_sql(path, &opts.embedded).with_context(|| {
            format!(
                "{RULE_ID} failed to collect embedded SQL facts from prepared analysis for {}",
                path.display()
            )
        })?;
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
        // Opaque executor text could take a FOR UPDATE lock. A recovered prefix
        // without a lock clause is treated as non-locking to keep reads quiet.
        if opts.fail_unanalyzable
            && call.kind == EmbeddedSqlKind::Dynamic
            && !has_safe_directive(source, call.line, "", &opts.safe_directive)
        {
            return vec![finding(
                file,
                call.line,
                unanalyzable_message(file, call.line, &opts.safe_directive),
                UNANALYZABLE_TARGET,
            )];
        }
        return Vec::new();
    };
    if !contains_for_update(sql) {
        return Vec::new();
    }
    if has_safe_directive(source, call.line, sql, &opts.safe_directive) {
        return Vec::new();
    }
    match extract_locking_select_metadata_with_placeholders(
        sql,
        &call.recovered_placeholder_positions,
    ) {
        Err(_) => vec![finding(
            file,
            call.line,
            unparseable_message(file, call.line, &opts.safe_directive),
            UNPARSEABLE_TARGET,
        )],
        Ok(locks) => lock_findings(file, call, opts, catalog, &locks),
    }
}

fn lock_findings(
    file: &str,
    call: &crate::codebase::postgres::EmbeddedSqlCall,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
    locks: &[LockingSelectMetadata],
) -> Vec<RuleFinding> {
    let directive = &opts.safe_directive;
    // A catalog unique key pinned by equality bounds the lock to one row, so an
    // `IN` / `= ANY` on another column is only a filter.
    let multi_row = |lock: &&LockingSelectMetadata| {
        lock.has_multi_row_predicate
            && !lock.skips_locked_rows
            && !catalog.is_some_and(|catalog| locks_single_row(lock, catalog))
    };
    if locks
        .iter()
        .filter(multi_row)
        .any(|lock| !lock.has_order_by)
    {
        return vec![finding(
            file,
            call.line,
            lock_ordering_message(file, call.line, directive),
            LOCK_ORDERING_TARGET,
        )];
    }
    let Some(catalog) = catalog else {
        return Vec::new();
    };
    let unordered: Vec<_> = locks
        .iter()
        .filter(multi_row)
        .filter(|lock| !orders_by_catalog_key(lock, catalog))
        .collect();
    if unordered.is_empty() {
        return Vec::new();
    }
    if !call.recovered_placeholder_positions.is_empty()
        && unordered
            .iter()
            .any(|lock| locks_interpolated_relation(lock))
    {
        return vec![finding(
            file,
            call.line,
            interpolated_relation_message(file, call.line, directive),
            UNRESOLVED_RELATION_TARGET,
        )];
    }
    vec![finding(
        file,
        call.line,
        canonical_order_message(file, call.line, directive),
        LOCK_ORDERING_TARGET,
    )]
}

fn interpolated_relation_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: multi-row FOR UPDATE locks a relation whose name is interpolated, so its schema-catalog key order cannot be checked; write the relation name literally, use SKIP LOCKED, or add a `{directive}` comment"
    )
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

fn unanalyzable_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: executed SQL is not statically recoverable, so a FOR UPDATE lock and its row order cannot be checked for ABBA deadlocks; pass a SQL literal or trusted tagged template, add a `{directive}` comment, or set unanalyzableSql: ignore"
    )
}
