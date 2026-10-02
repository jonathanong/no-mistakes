use super::super::parse::parse_postgres_sql_lenient;
use super::extract_dml_write_targets;
use sqlparser::ast::Statement;
use std::collections::{BTreeMap, BTreeSet};

mod insert;
pub(crate) mod names;
mod update;
pub(crate) mod width;

pub use insert::positional_insert_hits;

/// Generated columns keyed by qualified table identity, with unique unqualified lookup.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratedTableColumns {
    tables: BTreeMap<String, GeneratedTable>,
    relations: BTreeMap<String, BTreeSet<String>>,
    preferred: BTreeMap<String, String>,
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
        let key = catalog_key(&table.name);
        let base = crate::codebase::postgres::idents::relation_suffix_name(&key);
        self.register_relation(&key, &base);
        self.insert_table_with_key(&key, table);
    }

    pub(crate) fn register_relation(&mut self, key: &str, base: &str) {
        self.relations
            .entry(crate::codebase::postgres::idents::relation_part_key(base))
            .or_default()
            .insert(catalog_key(key));
    }

    pub(crate) fn display_name(&self, key: &str, base: &str) -> String {
        if self
            .relations
            .get(&crate::codebase::postgres::idents::relation_part_key(base))
            .is_some_and(|keys| keys.len() > 1)
            || key.contains('"')
        {
            key.to_string()
        } else {
            base.to_string()
        }
    }

    pub(crate) fn insert_table_with_key(&mut self, key: &str, table: GeneratedTable) {
        self.tables
            .entry(catalog_key(key))
            .and_modify(|existing| {
                existing.generated.extend(table.generated.iter().cloned());
                if existing.column_order.is_none() {
                    existing.column_order = table.column_order.clone();
                }
            })
            .or_insert(table);
    }

    pub(crate) fn extend_from(&mut self, other: &Self) {
        for (base, keys) in &other.relations {
            self.relations
                .entry(base.clone())
                .or_default()
                .extend(keys.iter().cloned());
        }
        self.preferred.extend(other.preferred.clone());
        for (key, table) in &other.tables {
            self.insert_table_with_key(key, table.clone());
        }
    }

    pub(crate) fn prefer_relation(&mut self, key: &str, base: &str) {
        self.preferred.insert(
            crate::codebase::postgres::idents::relation_part_key(base),
            catalog_key(key),
        );
    }

    pub(crate) fn get_exact(&self, table: &str) -> Option<&GeneratedTable> {
        self.tables.get(&catalog_key(table))
    }

    /// Resolve an exact relation or a unique unqualified name.
    pub fn get(&self, table: &str) -> Option<&GeneratedTable> {
        let key = catalog_key(table);
        if let Some(preferred) = self.preferred.get(&key) {
            return self.tables.get(preferred);
        }
        self.tables.get(&key).or_else(|| {
            let base = crate::codebase::postgres::idents::relation_suffix_key(&key);
            if base != key {
                // Preserve legacy public catalogs populated with unqualified names.
                return self.tables.get(base);
            }
            let keys = self.relations.get(&key)?;
            (keys.len() == 1)
                .then(|| self.tables.get(keys.first().unwrap()))
                .flatten()
        })
    }

    pub fn contains_table(&self, table: &str) -> bool {
        let key = catalog_key(table);
        self.tables.contains_key(&key)
            || self
                .relations
                .get(&key)
                .or_else(|| {
                    self.relations
                        .get(&crate::codebase::postgres::idents::relation_part_key(table))
                })
                .is_some_and(|keys| keys.iter().any(|key| self.tables.contains_key(key)))
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
    let sql = super::super::parse::expand_chr_encoded_sql(sql).unwrap_or_else(|| sql.to_string());
    let sql = sql.as_str();
    if !extract_dml_write_targets(sql)
        .iter()
        .any(|table| catalog.contains_table(table))
    {
        return Vec::new();
    }
    let statements = parse_postgres_sql_lenient(sql);
    let mut writes = Vec::new();
    for statement in &statements {
        collect_statement_writes(statement, catalog, &mut writes);
    }
    writes.sort_by(|left, right| {
        left.table.cmp(&right.table).then_with(|| {
            left.column
                .to_ascii_lowercase()
                .cmp(&right.column.to_ascii_lowercase())
        })
    });
    writes.dedup_by(|left, right| {
        left.table == right.table && left.column.eq_ignore_ascii_case(&right.column)
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

fn catalog_key(value: &str) -> String {
    if value.contains('"') {
        value.to_string()
    } else {
        value.to_ascii_lowercase()
    }
}
