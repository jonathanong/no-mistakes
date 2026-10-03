use super::{LiveColumn, LiveTable, LiveTables};
use crate::codebase::postgres::SqlColumnMetadata;

pub(super) fn create<'a>(
    tables: &mut LiveTables<'a>,
    key: &'a str,
    name: &'a str,
    columns: &'a [SqlColumnMetadata],
    if_not_exists: bool,
    temporary: bool,
    complete: bool,
) {
    let key = if temporary && !key.starts_with("pg_temp.") {
        format!("pg_temp.{key}")
    } else {
        key.to_string()
    };
    // IF NOT EXISTS is a no-op when an unqualified name resolves to a live relation.
    if if_not_exists && existing_key(tables, &key, name).is_some_and(|k| tables.contains_key(&k)) {
        return;
    }
    tables.insert(
        key,
        LiveTable {
            name,
            columns: columns
                .iter()
                .map(|column| LiveColumn {
                    name: &column.name,
                    generated: column.is_generated,
                })
                .collect(),
            complete,
            temporary,
        },
    );
}

pub(super) fn add<'a>(
    tables: &mut LiveTables<'a>,
    key: &'a str,
    table: &'a str,
    name: &'a str,
    generated: bool,
    if_not_exists: bool,
    table_if_exists: bool,
) {
    let Some(key) = existing_key(tables, key, table) else {
        return;
    };
    if table_if_exists && !tables.contains_key(&key) {
        return;
    }
    let entry = tables.entry(key.clone()).or_insert_with(|| LiveTable {
        name: table,
        columns: Vec::new(),
        complete: false,
        temporary: key.starts_with("pg_temp."),
    });
    let column = LiveColumn { name, generated };
    if let Some(existing) = entry.columns.iter_mut().find(|column| column.name == name) {
        if !if_not_exists {
            *existing = column;
        }
    } else {
        entry.columns.push(column);
    }
}

pub(super) fn existing_key(
    tables: &LiveTables<'_>,
    key: &str,
    unqualified: &str,
) -> Option<String> {
    let base = crate::codebase::postgres::idents::relation_part_key(unqualified);
    // An unqualified CREATE has unknown search_path; a later qualified ALTER can identify it.
    if key != base && !tables.contains_key(key) && tables.contains_key(&base) {
        return Some(base);
    }
    crate::codebase::postgres::idents::resolve_relation_key(tables.keys().map(String::as_str), key)
}
