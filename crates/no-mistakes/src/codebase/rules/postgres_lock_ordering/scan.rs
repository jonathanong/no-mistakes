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
mod messages;
mod versions;
use messages::{
    canonical_order_message, interpolated_relation_message, lock_ordering_message,
    unanalyzable_message, unparseable_message,
};

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
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
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
        let statements = file
            .calls
            .iter()
            .any(|call| !call.variants.is_empty())
            .then(|| facts.postgres_statements(path, Some(&opts.embedded)))
            .transpose()?;
        findings.extend(versions::findings_for_file(
            &rel, &source, file, opts, catalog, statements, &mut dedup,
        )?);
    }
    crate::codebase::rules::sort_postgres_findings(&mut findings);
    Ok(findings)
}

pub(super) fn findings_for_call_with_catalog(
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

fn finding(file: &str, line: u32, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        source_offset: None,
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: line as usize,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
