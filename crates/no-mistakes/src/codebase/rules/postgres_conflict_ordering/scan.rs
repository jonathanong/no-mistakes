use crate::codebase::check_facts::SqlFactContext;
mod analysis;
mod exact;
mod gate;
mod sql_sources;
mod substitute;
mod verdicts;

use super::{CompiledOptions, RULE_ID};
use crate::codebase::postgres::{
    recovered_sql_needs_insert_check, EmbeddedSqlCall, EmbeddedSqlKind,
};
use crate::codebase::rules::postgres_lock_ordering::directive::has_safe_directive;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const OPAQUE_SQL_MESSAGE: &str =
    "keep executor SQL statically recoverable so canonical ON CONFLICT ordering can be checked";
const DYNAMIC_INSERT_MESSAGE: &str =
    "keep dynamic INSERT SQL statically parseable so canonical ON CONFLICT ordering can be checked";

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &dyn SqlFactContext,
) -> Result<crate::codebase::rules::PostgresFindings> {
    let prepared = facts.prepared();
    let catalog = prepared.postgres_ordering_catalog(&opts.schema_catalog_path)?;
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let file = prepared
            .embedded_sql(path, &opts.embedded)
            .with_context(|| {
                format!(
                    "{RULE_ID} failed to collect embedded SQL facts from prepared analysis for {}",
                    path.display()
                )
            })?;
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path).unwrap_or_default();
        let statements = crate::codebase::rules::index_sql_variants(
            prepared.postgres_statements(path, Some(&opts.embedded))?,
        );
        if facts.wants_sql_verdicts() {
            verdicts::record(&rel, &source, file, opts, catalog, &statements, facts);
        }
        let mut residual_reported = None;
        for (call_index, variant_index, original, call) in
            file.calls
                .iter()
                .enumerate()
                .flat_map(|(call_index, original)| {
                    original
                        .statement_calls()
                        .enumerate()
                        .map(move |(variant_index, call)| {
                            (call_index, variant_index, original, call)
                        })
                })
        {
            if has_safe_directive(
                &source,
                call.line,
                call.sql_text.as_deref().unwrap_or_default(),
                &opts.safe_directive,
            ) {
                continue;
            }
            let sql = call.sql_text.as_deref();
            // Retained alternatives are complete statements even when more
            // outcomes remain. Check them, and report the residual separately.
            if should_report_residual(
                opts.fail_unanalyzable,
                residual_reported == Some(call_index),
                original,
                &call,
            ) {
                residual_reported = Some(call_index);
                findings.push(unanalyzable_sql(&rel, call.line, DYNAMIC_INSERT_MESSAGE));
            }
            if call.kind == EmbeddedSqlKind::Dynamic && call.variants.is_empty() {
                if opts.fail_unanalyzable && recovered_sql_needs_insert_check(sql) {
                    findings.push(unanalyzable_sql(
                        &rel,
                        call.line,
                        if sql.is_none() {
                            OPAQUE_SQL_MESSAGE
                        } else {
                            DYNAMIC_INSERT_MESSAGE
                        },
                    ));
                }
                continue;
            }
            let Some(sql) = sql else {
                if opts.fail_unanalyzable {
                    findings.push(unanalyzable_sql(&rel, call.line, OPAQUE_SQL_MESSAGE));
                }
                continue;
            };
            // Conflict-only preparation omits exact statements without a possible
            // INSERT conflict, including ordinary INSERTs. Dynamic SQL stays above.
            if !crate::codebase::postgres::may_have_insert_conflict(sql) {
                continue;
            }
            let statement = statements.get(&(call_index, variant_index)).copied();
            let has_prepared_conflict = statement
                .and_then(|statement| statement.variant_locations.as_ref())
                .is_some_and(|locations| {
                    !locations.conflicts.is_empty() || locations.conflict_failure_overlaps
                });
            // A data-modifying CTE can execute INSERT while its outer statement
            // is SELECT; a failed SELECT prefix can also precede a broken INSERT.
            // Both prepared conflict facts still require this policy.
            let missing_conflict = statement.is_none() && analysis::contains_insert_conflict(sql);
            if !recovered_sql_needs_insert_check(Some(sql))
                && !has_prepared_conflict
                && !missing_conflict
            {
                continue;
            }
            let statement = statement.ok_or_else(|| {
                let missing = if original.variants.is_empty() {
                    "prepared SQL facts are missing"
                } else {
                    "prepared SQL variant facts are missing"
                };
                anyhow::anyhow!("{RULE_ID}: {missing} for {rel}:{}", call.line)
            })?;
            if original.variants.is_empty() {
                findings.extend(exact::findings(
                    &rel,
                    call.line as usize,
                    sql,
                    statement,
                    catalog,
                    opts.fail_unanalyzable,
                )?);
                continue;
            }
            let locations = exact::locations(statement, &rel, call.line as usize)?;
            let prepared = gate::prepared_conflicts(locations, statement.parse_failed);
            if prepared.unanalyzable
                && opts.fail_unanalyzable
                && analysis::contains_insert_conflict(sql)
            {
                dedup.push(&mut findings, statement, crate::codebase::postgres::statements::SqlFactSite::Origin, Some(&source), analysis::finding(&rel, call.line as usize, "unanalyzable-sql", "keep INSERT ... ON CONFLICT SQL statically parseable so canonical ordering can be checked"));
            }
            for (index, finding) in analysis::findings_for_inserts(
                &rel,
                call.line as usize,
                prepared.conflicts,
                catalog,
            ) {
                dedup.push(
                    &mut findings,
                    statement,
                    crate::codebase::postgres::statements::SqlFactSite::Conflict(index),
                    Some(&source),
                    finding,
                );
            }
        }
    }
    sql_sources::scan(root, opts, files, sources, catalog, &mut findings)?;
    findings.sort();
    Ok(findings)
}

// Report once on a non-exempt INSERT preview; cheap guards precede variant scans.
fn should_report_residual(
    fail_unanalyzable: bool,
    already_reported: bool,
    original: &EmbeddedSqlCall,
    projected: &EmbeddedSqlCall,
) -> bool {
    fail_unanalyzable
        && !already_reported
        && original.variants_truncated()
        && recovered_sql_needs_insert_check(projected.sql_text.as_deref())
}

fn unanalyzable_sql(rel: &str, line: u32, message: &'static str) -> RuleFinding {
    analysis::finding(rel, line as usize, "unanalyzable-sql", message)
}

#[cfg(test)]
mod tests;
