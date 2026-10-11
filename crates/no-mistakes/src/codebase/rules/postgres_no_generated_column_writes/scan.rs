use super::{finding, trigger_finding, unanalyzable_finding, CompiledOptions};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::postgres::dml::writes::positional_insert_hits;
use crate::codebase::postgres::{
    recovered_sql_may_write_columns, EmbeddedSqlFileFacts, SqlStatementFileFacts, SqlWriteColumns,
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use history::{events_before, snapshots, Catalogs};
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    let schema_files = crate::codebase::postgres::postgres_sql_paths(root, files, &opts.schema)
        .map_err(|error| {
            anyhow::anyhow!(
                "{} failed to collect PostgreSQL facts: {error}",
                super::RULE_ID
            )
        })?;
    let schema: Vec<_> = schema_files
        .iter()
        .map(|path| facts.postgres_readable_schema(path))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    let tables = super::catalog::live_tables(&schema);
    let finals = Catalogs::build(&tables, opts);
    let mut findings =
        super::catalog::stale_extra_findings_from_tables(&tables, &opts.extra_generated_columns);
    findings.extend(super::catalog::stale_trigger_findings(
        &tables,
        &opts.trigger_maintained_columns,
    ));
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
    let has_history = schema.iter().any(|file| !file.table_events.is_empty());
    if !finals.combined.is_empty() || has_history {
        for path in files {
            if !opts.includes_dml(root, path) {
                continue;
            }
            let profile = is_indexable(path).then_some(&opts.embedded);
            if profile.is_none() && path.extension().and_then(|ext| ext.to_str()) != Some("sql") {
                continue;
            }
            // Source/TS parse failures keep the rule's existing per-file tolerance.
            let Some(statements) = facts.postgres_readable_statements(path, profile)? else {
                continue;
            };
            // Migrations that define schema are matched against the catalog as of each write.
            let history = schema
                .iter()
                .find(|file| file.path == *path && !file.table_events.is_empty());
            let snapshots = history
                .map(|file| snapshots(&schema, file, statements, opts))
                .unwrap_or_default();
            let lookup = |line: usize| match history {
                Some(file) => snapshots.get(&events_before(file, line)).unwrap_or(&finals),
                None => &finals,
            };
            let rel = relative_slash_path(root, path);
            let source = crate::codebase::rules::read_source(sources, path);
            extend_writes(
                &mut findings,
                &rel,
                statements,
                &mut dedup,
                source.as_deref(),
                &lookup,
            );
            if let Some(embedded) = profile
                .filter(|_| opts.fail_unanalyzable)
                .and_then(|profile| facts.embedded_sql(path, profile).ok())
            {
                extend_unanalyzable(&mut findings, &rel, embedded, statements, &lookup);
            }
        }
    }
    crate::codebase::rules::sort_postgres_findings(&mut findings);
    Ok(findings)
}

/// An opaque tail may introduce another write even when recovered targets
/// are literal and untracked. Only an empty applicable catalog is safe.
fn extend_unanalyzable<'a>(
    findings: &mut Vec<RuleFinding>,
    file: &str,
    embedded: &EmbeddedSqlFileFacts,
    _statements: &[SqlStatementFileFacts],
    lookup: &dyn Fn(usize) -> &'a Catalogs,
) {
    for call in &embedded.calls {
        let line = call.line.max(1) as usize;
        if call.is_unanalyzable()
            && recovered_sql_may_write_columns(call.sql_text.as_deref())
            && !lookup(line).combined.is_empty()
        {
            findings.push(unanalyzable_finding(file, line));
        }
    }
}

fn extend_writes<'a>(
    findings: &mut Vec<RuleFinding>,
    file: &str,
    statements: &[SqlStatementFileFacts],
    dedup: &mut crate::codebase::rules::VariantFindingDedup,
    source: Option<&str>,
    lookup: &dyn Fn(usize) -> &'a Catalogs,
) {
    for statement in statements {
        let mut hits = Vec::new();
        for (write_index, write) in statement.writes.iter().enumerate() {
            let catalog = &lookup(write.line).combined;
            let Some(meta) = catalog.get(&write.table) else {
                continue;
            };
            let columns: Vec<String> = match &write.columns {
                SqlWriteColumns::Named(columns) => columns
                    .iter()
                    .filter(|column| meta.generated.contains(&column.to_ascii_lowercase()))
                    .cloned()
                    .collect(),
                SqlWriteColumns::Positional(width) => {
                    positional_insert_hits(&write.table, meta, *width)
                        .into_iter()
                        .map(|hit| hit.column)
                        .collect()
                }
                SqlWriteColumns::All => meta.generated.iter().cloned().collect(),
            };
            for column in columns {
                hits.push((write.line, meta.name.as_str(), column, write_index));
            }
        }
        hits.sort();
        hits.dedup();
        for (line, table, column, write_index) in hits {
            let generated = &lookup(line).catalog;
            let render = if generated
                .get_exact(table)
                .or_else(|| generated.get(table))
                .is_some_and(|meta| meta.generated.contains(&column.to_ascii_lowercase()))
            {
                finding
            } else {
                trigger_finding
            };
            dedup.push(
                findings,
                statement,
                write_site(statement, write_index, &column),
                source,
                render(file, line, table, &column),
            );
        }
    }
}

mod history;
#[cfg(test)]
mod tests;

fn write_site(
    statement: &SqlStatementFileFacts,
    index: usize,
    column: &str,
) -> crate::codebase::postgres::statements::SqlFactSite {
    use crate::codebase::postgres::statements::SqlFactSite;
    let named = SqlFactSite::WriteColumn(index, column.to_string());
    if statement
        .variant_locations
        .as_ref()
        .is_some_and(|locations| locations.position(named.clone()).is_some())
    {
        named
    } else {
        SqlFactSite::Write(index)
    }
}
