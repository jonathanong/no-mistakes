use crate::codebase::postgres::{SqlColumnMetadata, SqlSchemaFileFacts, SqlTableSchemaEvent};
use std::collections::BTreeMap;

pub(crate) struct LiveColumn<'a> {
    pub(super) name: &'a str,
    pub(super) generated: bool,
}

pub(crate) struct LiveTable<'a> {
    pub(super) name: &'a str,
    pub(super) columns: Vec<LiveColumn<'a>>,
    pub(super) complete: bool,
}

pub(crate) type LiveTables<'a> = BTreeMap<String, LiveTable<'a>>;

pub(crate) fn live_tables(schema: &[SqlSchemaFileFacts]) -> LiveTables<'_> {
    let mut tables = BTreeMap::new();
    for file in schema {
        if file.table_events.is_empty() {
            // Public Rust callers can supply the original facts without ordered events.
            for table in &file.tables {
                create(&mut tables, &table.table_name, &table.columns, false);
            }
            for column in &file.add_columns {
                add(
                    &mut tables,
                    &column.unqualified_table_name,
                    &column.column_name,
                    column.is_generated,
                    false,
                );
            }
            continue;
        }
        for event in &file.table_events {
            match event {
                SqlTableSchemaEvent::Create {
                    unqualified_table,
                    columns,
                    if_not_exists,
                    ..
                } => {
                    create(&mut tables, unqualified_table, columns, *if_not_exists);
                }
                SqlTableSchemaEvent::AddColumn {
                    unqualified_table,
                    column,
                    if_not_exists,
                    ..
                } => {
                    add(
                        &mut tables,
                        unqualified_table,
                        &column.name,
                        column.is_generated,
                        *if_not_exists,
                    );
                }
                SqlTableSchemaEvent::Drop {
                    unqualified_table, ..
                } => {
                    tables.remove(&unqualified_table.to_ascii_lowercase());
                }
            }
        }
    }
    tables
}

fn create<'a>(
    tables: &mut LiveTables<'a>,
    name: &'a str,
    columns: &'a [SqlColumnMetadata],
    if_not_exists: bool,
) {
    if if_not_exists && tables.contains_key(&name.to_ascii_lowercase()) {
        return;
    }
    tables.insert(
        name.to_ascii_lowercase(),
        LiveTable {
            name,
            columns: columns
                .iter()
                .map(|column| LiveColumn {
                    name: &column.name,
                    generated: column.is_generated,
                })
                .collect(),
            complete: true,
        },
    );
}

fn add<'a>(
    tables: &mut LiveTables<'a>,
    table: &'a str,
    name: &'a str,
    generated: bool,
    if_not_exists: bool,
) {
    let entry = tables
        .entry(table.to_ascii_lowercase())
        .or_insert_with(|| LiveTable {
            name: table,
            columns: Vec::new(),
            complete: false,
        });
    let column = LiveColumn { name, generated };
    if let Some(existing) = entry
        .columns
        .iter_mut()
        .find(|column| column.name.eq_ignore_ascii_case(name))
    {
        if !if_not_exists {
            *existing = column;
        }
    } else {
        entry.columns.push(column);
    }
}
