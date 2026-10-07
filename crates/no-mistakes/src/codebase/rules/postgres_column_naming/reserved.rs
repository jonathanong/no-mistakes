use super::compile::{lists_type, Compiled, ReservedCheck};
use super::follow::leads_to;
use super::singular::join_or;
use crate::codebase::postgres::{CatalogColumn, CatalogForeignKey, CatalogTable, SchemaCatalog};

pub(super) fn reserved_texts(
    column: &CatalogColumn,
    table: &CatalogTable,
    catalog: &SchemaCatalog,
    compiled: &Compiled,
) -> Vec<String> {
    compiled
        .reserved
        .iter()
        .filter(|entry| hits(&column.name, &entry.suffix) && type_ok(&column.data_type, entry))
        .filter(|entry| !satisfied(column, table, catalog, compiled, entry))
        .map(|entry| text(column, table, entry, compiled.follow))
        .collect()
}

pub(super) fn hits(name: &str, suffix: &str) -> bool {
    name.ends_with(suffix) || suffix.strip_prefix('_').is_some_and(|bare| name == bare)
}

fn type_ok(data_type: &str, entry: &ReservedCheck) -> bool {
    entry.types.is_empty() || lists_type(&entry.types, data_type)
}

fn satisfied(
    column: &CatalogColumn,
    table: &CatalogTable,
    catalog: &SchemaCatalog,
    compiled: &Compiled,
    entry: &ReservedCheck,
) -> bool {
    let sole = table.foreign_keys.iter().any(|foreign_key| {
        sole(foreign_key, &column.name)
            && entry
                .tables
                .iter()
                .any(|name| name == &foreign_key.referenced_table)
    });
    sole || (compiled.follow && leads_to(catalog, &table.name, &column.name, &entry.tables))
}

fn text(
    column: &CatalogColumn,
    table: &CatalogTable,
    entry: &ReservedCheck,
    follow: bool,
) -> String {
    let clause = if let Some(foreign_key) = first_sole_other(table, &column.name, &entry.tables) {
        format!("this column references {}", foreign_key.referenced_table)
    } else if let Some(foreign_key) = first_composite(table, &column.name) {
        composite_clause(foreign_key, entry, follow)
    } else {
        format!("this {} column has no foreign key", column.data_type)
    };
    let mut message = format!(
        "{} is reserved for foreign keys to {}; {clause}",
        entry.suffix,
        join_or(&entry.tables)
    );
    if let Some(hint) = &entry.hint {
        message.push_str("; ");
        message.push_str(hint);
    }
    message
}

fn composite_clause(
    foreign_key: &CatalogForeignKey,
    entry: &ReservedCheck,
    follow: bool,
) -> String {
    if follow {
        format!(
            "this column is part of a composite foreign key to {}, which does not lead to {}",
            foreign_key.referenced_table,
            join_or(&entry.tables)
        )
    } else {
        format!(
            "this column is only part of a composite foreign key to {}",
            foreign_key.referenced_table
        )
    }
}

fn first_sole_other<'a>(
    table: &'a CatalogTable,
    column: &str,
    tables: &[String],
) -> Option<&'a CatalogForeignKey> {
    table.foreign_keys.iter().find(|foreign_key| {
        sole(foreign_key, column)
            && !tables
                .iter()
                .any(|name| name == &foreign_key.referenced_table)
    })
}

fn first_composite<'a>(table: &'a CatalogTable, column: &str) -> Option<&'a CatalogForeignKey> {
    table.foreign_keys.iter().find(|foreign_key| {
        foreign_key.columns.len() > 1 && foreign_key.columns.iter().any(|name| name == column)
    })
}

fn sole(foreign_key: &CatalogForeignKey, column: &str) -> bool {
    foreign_key.columns.len() == 1 && foreign_key.columns[0] == column
}
