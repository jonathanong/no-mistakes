use super::{CompiledOptions, RuleFinding, SourceStore, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::statements::{SqlFactSite, SqlJsonWriteFact};
use crate::codebase::postgres::{CatalogTable, SchemaCatalog, SqlWriteColumns};
use crate::codebase::rules::{PostgresFindings, VariantFindingDedup};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
) -> Result<PostgresFindings> {
    let catalog = (!opts.schema_catalog_path.trim().is_empty())
        .then(|| facts.postgres_schema_catalog(&opts.schema_catalog_path))
        .transpose()?;
    let mut findings = PostgresFindings::default();
    if let Some(catalog) = catalog {
        findings.extend(super::fingerprint::scan(opts, catalog));
    }
    if opts.id_key_pattern.is_none() {
        return Ok(findings);
    }
    // Literal JSON documents give every key the same SQL token origin. Keep
    // separate key identities while still deduplicating repeated branch variants.
    let mut dedup = std::collections::BTreeMap::new();
    for path in files {
        let rel = relative_slash_path(root, path);
        let profile = (!rel.ends_with(".sql")).then_some(&opts.embedded);
        let statements = match facts.postgres_statements(path, profile) {
            Ok(statements) => statements,
            Err(_) if profile.is_none() && facts.postgres.sql_source_not_found(path) => continue,
            Err(error) => return Err(error),
        };
        let source = sources.read_path(path)?;
        for statement in statements {
            for (index, write) in statement.json_writes.iter().enumerate() {
                if let Some(finding) = json_finding(&rel, opts, catalog, write) {
                    dedup
                        .entry(&write.key)
                        .or_insert_with(|| VariantFindingDedup::for_root(root, sources))
                        .push(
                            &mut findings,
                            statement,
                            SqlFactSite::JsonWrite(index),
                            Some(&source),
                            finding,
                        );
                }
            }
        }
    }
    findings.sort();
    Ok(findings)
}

fn json_finding(
    file: &str,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
    write: &SqlJsonWriteFact,
) -> Option<RuleFinding> {
    if !opts.id_key_pattern.as_ref()?.is_match(&write.key) {
        return None;
    }
    let table = catalog.and_then(|catalog| catalog.relation(&write.table));
    let table_name = table.map_or(write.table.as_str(), |table| table.name.as_str());
    let column = write.column.as_deref().or_else(|| {
        table.and_then(|table| {
            write
                .column_index
                .and_then(|index| positional_column(table, index))
        })
    });
    if opts.allows(table_name, column.unwrap_or_default()) {
        return None;
    }
    let mapped = opts.key_column_map.get(&write.key);
    if mapped.is_some_and(|mapped| {
        table.is_some_and(|table| {
            table
                .foreign_keys
                .iter()
                .any(|key| key.columns.contains(mapped))
                && writes_column(write, table, mapped)
        })
    }) {
        return None;
    }
    let mapped = mapped.map_or("a mapped column", String::as_str);
    let line = write.line.max(1);
    // A literal key stays decidable even without a catalog/explicit column list.
    // Do not turn a missing target-column name into a silent omission.
    let location = column.map_or_else(
        || {
            write.column_index.map_or_else(
                || format!("an unresolved column of {table_name}"),
                |index| format!("positional column {} of {table_name}", index + 1),
            )
        },
        |column| format!("{table_name}.{column}"),
    );
    let target = column.map_or_else(
        || format!("table:{table_name}"),
        |column| format!("column:{table_name}.{column}"),
    );
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: key {:?} written into {location} has no same-row foreign key column; write {mapped} with a foreign key, or allow the column with a reason", write.key),
        import: None,
        target: Some(target),
    })
}

fn positional_column(table: &CatalogTable, index: usize) -> Option<&str> {
    table
        .columns
        .iter()
        .find(|column| column.ordinal_position as usize == index + 1)
        .map(|column| column.name.as_str())
}

fn writes_column(write: &SqlJsonWriteFact, table: &CatalogTable, mapped: &str) -> bool {
    match &write.written_columns {
        SqlWriteColumns::Named(columns) => columns.iter().any(|column| column == mapped),
        SqlWriteColumns::Positional(Some(width)) => table.columns.iter().any(|column| {
            column.name == mapped
                && column.ordinal_position > 0
                && column.ordinal_position as usize <= *width
        }),
        _ => false,
    }
}
