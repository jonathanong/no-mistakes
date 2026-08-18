use super::super::parse::parse_postgres_sql;
use super::extract_dml_write_targets;
use sqlparser::ast::Statement;
use std::collections::{BTreeMap, BTreeSet};

mod insert;
mod names;
mod update;
mod width;

pub use insert::positional_insert_hits;

/// Schema or extra generated columns keyed by lowercase table name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratedTableColumns {
    tables: BTreeMap<String, GeneratedTable>,
}

/// Generated columns for one table, plus CREATE TABLE order when known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTable {
    pub name: String,
    pub generated: BTreeSet<String>,
    pub column_order: Option<Vec<String>>,
}

/// One parsed DML write of a generated column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedColumnWrite {
    pub table: String,
    pub column: String,
}

impl GeneratedTableColumns {
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    pub fn insert_table(&mut self, table: GeneratedTable) {
        let key = table.name.to_ascii_lowercase();
        self.tables
            .entry(key)
            .and_modify(|existing| {
                existing.generated.extend(table.generated.iter().cloned());
                if existing.column_order.is_none() {
                    existing.column_order = table.column_order.clone();
                }
            })
            .or_insert(table);
    }

    pub fn get(&self, table: &str) -> Option<&GeneratedTable> {
        self.tables.get(&table.to_ascii_lowercase())
    }

    pub fn contains_table(&self, table: &str) -> bool {
        self.tables.contains_key(&table.to_ascii_lowercase())
    }
}

/// Parse `sql` only when the cheap table prefilter hits a generated table.
pub fn find_generated_column_writes(
    sql: &str,
    catalog: &GeneratedTableColumns,
) -> Vec<GeneratedColumnWrite> {
    if catalog.is_empty() {
        return Vec::new();
    }
    if !extract_dml_write_targets(sql)
        .iter()
        .any(|table| catalog.contains_table(table))
    {
        return Vec::new();
    }
    let Ok(statements) = parse_postgres_sql(sql) else {
        return Vec::new();
    };
    let mut writes = Vec::new();
    for statement in &statements {
        collect_statement_writes(statement, catalog, &mut writes);
    }
    writes.sort_by(|left, right| {
        left.table
            .to_ascii_lowercase()
            .cmp(&right.table.to_ascii_lowercase())
            .then_with(|| {
                left.column
                    .to_ascii_lowercase()
                    .cmp(&right.column.to_ascii_lowercase())
            })
    });
    writes.dedup_by(|left, right| {
        left.table.eq_ignore_ascii_case(&right.table)
            && left.column.eq_ignore_ascii_case(&right.column)
    });
    writes
}

fn collect_statement_writes(
    statement: &Statement,
    catalog: &GeneratedTableColumns,
    writes: &mut Vec<GeneratedColumnWrite>,
) {
    match statement {
        Statement::Update(update) => update::collect_update_writes(update, catalog, writes),
        Statement::Insert(insert) => insert::collect_insert_writes(insert, catalog, writes),
        Statement::Merge(merge) => update::collect_merge_writes(merge, catalog, writes),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
