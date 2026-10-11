mod analysis;
mod substitute;

use super::{CompiledOptions, RULE_ID};
use crate::codebase::postgres::{
    postgres_sql_paths, recovered_sql_needs_insert_check, EmbeddedSqlKind,
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
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let catalog = facts.postgres_ordering_catalog(&opts.schema_catalog_path)?;
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
            .transpose()?
            .map(crate::codebase::rules::index_sql_variants)
            .unwrap_or_default();
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
            if call.kind == EmbeddedSqlKind::Dynamic {
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
            if !recovered_sql_needs_insert_check(Some(sql)) {
                continue;
            }
            if original.variants.is_empty() {
                findings.extend(analysis::findings_for_sql_with_binds(
                    &rel,
                    call.line as usize,
                    sql,
                    &call.recovered_placeholder_positions,
                    catalog,
                    opts.fail_unanalyzable,
                ));
                continue;
            }
            let statement = statements
                .get(&(call_index, variant_index))
                .copied()
                .ok_or_else(|| anyhow::anyhow!("prepared SQL variant facts are missing"))?;
            let locations = statement.variant_locations.as_ref().unwrap();
            if locations.conflict_error.is_some()
                && opts.fail_unanalyzable
                && analysis::contains_insert_conflict(sql)
            {
                dedup.push(&mut findings, statement, crate::codebase::postgres::statements::SqlFactSite::Origin, Some(&source), analysis::finding(&rel, call.line as usize, "unanalyzable-sql", "keep INSERT ... ON CONFLICT SQL statically parseable so canonical ordering can be checked"));
            }
            for (index, finding) in analysis::findings_for_inserts(
                &rel,
                call.line as usize,
                &locations.conflicts,
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
    if let Some(sql_sources) = &opts.sql_sources {
        for path in postgres_sql_paths(root, files, sql_sources)? {
            let rel = relative_slash_path(root, &path);
            let source = crate::codebase::rules::read_source(sources, &path).unwrap_or_default();
            for (line, statement) in analysis::sql_statements(&source) {
                if has_safe_directive(statement, 1, statement, &opts.safe_directive) {
                    continue;
                }
                findings.extend(analysis::findings_for_sql(
                    &rel,
                    line as usize,
                    statement,
                    catalog,
                    opts.fail_unanalyzable,
                ));
            }
        }
    }
    crate::codebase::rules::sort_postgres_findings(&mut findings);
    Ok(findings)
}

fn unanalyzable_sql(rel: &str, line: u32, message: &'static str) -> RuleFinding {
    analysis::finding(rel, line as usize, "unanalyzable-sql", message)
}
