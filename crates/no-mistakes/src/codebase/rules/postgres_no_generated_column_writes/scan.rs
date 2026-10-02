use super::{finding, trigger_finding, CompiledOptions};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::postgres::dml::writes::positional_insert_hits;
use crate::codebase::postgres::dml::GeneratedTableColumns;
use crate::codebase::postgres::{SqlStatementFileFacts, SqlWriteColumns};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
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
    let catalog = super::catalog::catalog_from_tables(&tables, &opts.extra_generated_columns);
    let trigger =
        super::catalog::trigger_catalog_from_tables(&tables, &opts.trigger_maintained_columns);
    let mut combined = trigger.clone();
    combined.extend_from(&catalog);
    let mut findings =
        super::catalog::stale_extra_findings_from_tables(&tables, &opts.extra_generated_columns);
    findings.extend(super::catalog::stale_trigger_findings(
        &tables,
        &opts.trigger_maintained_columns,
    ));
    if !combined.is_empty() {
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
            extend_writes(
                &mut findings,
                &relative_slash_path(root, path),
                statements,
                &combined,
                &catalog,
            );
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn extend_writes(
    findings: &mut Vec<RuleFinding>,
    file: &str,
    statements: &[SqlStatementFileFacts],
    catalog: &GeneratedTableColumns,
    generated: &GeneratedTableColumns,
) {
    for statement in statements {
        let mut hits = Vec::new();
        for write in &statement.writes {
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
                hits.push((write.line, meta.name.as_str(), column));
            }
        }
        hits.sort();
        hits.dedup();
        for (line, table, column) in hits {
            let render = if generated
                .get_exact(table)
                .or_else(|| generated.get(table))
                .is_some_and(|meta| meta.generated.contains(&column.to_ascii_lowercase()))
            {
                finding
            } else {
                trigger_finding
            };
            findings.push(render(file, line, table, &column));
        }
    }
}

#[cfg(test)]
mod tests;
