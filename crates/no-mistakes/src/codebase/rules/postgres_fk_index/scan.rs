use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, SqlCreateIndexMetadata, SqlForeignKeyMetadata, SqlSchemaFileFacts,
};
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_schema: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &Default::default(),
    )
    .with_context(|| format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let indexes = indexes_by_table(&facts.schema);
    let mut used_columns = BTreeSet::new();
    let mut used_tables = BTreeSet::new();
    let mut findings = Vec::new();
    for file in &facts.schema {
        let rel = sql_rel(root, &file.path);
        let source = sources
            .read_path(&file.path)
            .map(|source| source.to_string())
            .unwrap_or_default();
        for fk in &file.foreign_keys {
            findings.extend(scan_fk(
                &rel,
                &source,
                fk,
                &indexes,
                opts,
                &mut used_columns,
                &mut used_tables,
            ));
        }
    }
    findings.extend(stale_allowlist(opts, &used_columns, &used_tables));
    Ok(findings)
}

pub(super) fn scan_fk(
    rel: &str,
    source: &str,
    fk: &SqlForeignKeyMetadata,
    indexes: &BTreeMap<String, Vec<&SqlCreateIndexMetadata>>,
    opts: &CompiledOptions,
    used_columns: &mut BTreeSet<String>,
    used_tables: &mut BTreeSet<String>,
) -> Vec<RuleFinding> {
    let Some(column) = fk.column_names.first() else {
        return Vec::new();
    };
    if opts.allowed_tables.contains(&fk.table_name) {
        used_tables.insert(fk.table_name.clone());
        return Vec::new();
    }
    let table_indexes = indexes
        .get(&fk.table_name)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if table_indexes.iter().any(|index| covers(index, column)) {
        return Vec::new();
    }
    if directive_on_line(source, fk.line, &opts.allow_directive) {
        return Vec::new();
    }
    let key = format!("{}.{}", fk.table_name, column);
    if opts.allowed_columns.contains(&key) {
        used_columns.insert(key);
        return Vec::new();
    }
    vec![RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line: fk.line.max(1),
        message: format!(
            "{key}: foreign key has no leading btree/hash index (predicate must be absent or `WHERE {column} IS NOT NULL`)"
        ),
        import: None,
        target: Some(key),
    }]
}

pub(super) fn covers(index: &SqlCreateIndexMetadata, column: &str) -> bool {
    let leading = index.leading_column.as_deref();
    if !leading.is_some_and(|name| name.eq_ignore_ascii_case(column)) {
        return false;
    }
    if !matches!(index.access_method.as_str(), "btree" | "hash") {
        return false;
    }
    if !index.has_predicate {
        return true;
    }
    index
        .not_null_predicate_column
        .as_deref()
        .is_some_and(|name| name.eq_ignore_ascii_case(column))
}

fn indexes_by_table(
    schema: &[SqlSchemaFileFacts],
) -> BTreeMap<String, Vec<&SqlCreateIndexMetadata>> {
    let mut indexes = BTreeMap::<String, Vec<&SqlCreateIndexMetadata>>::new();
    for file in schema {
        for index in &file.indexes {
            indexes
                .entry(index.table_name.clone())
                .or_default()
                .push(index);
        }
    }
    indexes
}

pub(super) fn directive_on_line(source: &str, line: usize, directive: &str) -> bool {
    if directive.is_empty() || line == 0 {
        return false;
    }
    source
        .lines()
        .nth(line - 1)
        .is_some_and(|text| text.contains("--") && text.contains(directive))
}

fn stale_allowlist(
    opts: &CompiledOptions,
    used_columns: &BTreeSet<String>,
    used_tables: &BTreeSet<String>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for column in &opts.allowed_columns {
        if used_columns.contains(column) {
            continue;
        }
        findings.push(stale("allowedColumns", column));
    }
    for table in &opts.allowed_tables {
        if used_tables.contains(table) {
            continue;
        }
        findings.push(stale("allowedTables", table));
    }
    findings
}

fn stale(kind: &str, entry: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: entry.to_string(),
        line: 1,
        message: format!("stale postgres-fk-index {kind} entry: {entry}"),
        import: None,
        target: Some(entry.to_string()),
    }
}
