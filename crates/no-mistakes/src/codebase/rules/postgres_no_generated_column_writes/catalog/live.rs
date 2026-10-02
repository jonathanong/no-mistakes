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
    let mut ordered: Vec<_> = schema.iter().collect();
    ordered.sort_by(|left, right| {
        crate::codebase::postgres::cmp_sql_rel(
            &left.path.to_string_lossy().replace('\\', "/"),
            &right.path.to_string_lossy().replace('\\', "/"),
        )
    });
    for file in ordered {
        if !file.table_events_collected {
            // Public Rust callers can supply the original facts without ordered events.
            for table in &file.tables {
                create(
                    &mut tables,
                    &table.table_name,
                    &table.table_name,
                    &table.columns,
                    false,
                );
            }
            for column in &file.add_columns {
                add(
                    &mut tables,
                    &column.table_name,
                    &column.unqualified_table_name,
                    &column.column_name,
                    column.is_generated,
                    false,
                    false,
                );
            }
            continue;
        }
        for event in &file.table_events {
            match event {
                SqlTableSchemaEvent::Create {
                    table,
                    unqualified_table,
                    columns,
                    if_not_exists,
                    ..
                } => {
                    create(
                        &mut tables,
                        table,
                        unqualified_table,
                        columns,
                        *if_not_exists,
                    );
                }
                SqlTableSchemaEvent::AddColumn {
                    table,
                    unqualified_table,
                    column,
                    if_not_exists,
                    table_if_exists,
                    ..
                } => {
                    add(
                        &mut tables,
                        table,
                        unqualified_table,
                        &column.name,
                        column.is_generated,
                        *if_not_exists,
                        *table_if_exists,
                    );
                }
                SqlTableSchemaEvent::Drop {
                    table,
                    unqualified_table,
                    ..
                } => {
                    let key = existing_key(&tables, table, unqualified_table).to_string();
                    tables.remove(&key);
                }
            }
        }
    }
    tables
}

fn create<'a>(
    tables: &mut LiveTables<'a>,
    key: &'a str,
    name: &'a str,
    columns: &'a [SqlColumnMetadata],
    if_not_exists: bool,
) {
    if if_not_exists && tables.contains_key(key) {
        return;
    }
    tables.insert(
        key.to_string(),
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
    key: &'a str,
    table: &'a str,
    name: &'a str,
    generated: bool,
    if_not_exists: bool,
    table_if_exists: bool,
) {
    let key = existing_key(tables, key, table).to_string();
    if table_if_exists && !tables.contains_key(&key) {
        return;
    }
    let entry = tables.entry(key).or_insert_with(|| LiveTable {
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

fn existing_key<'a>(tables: &LiveTables<'_>, key: &'a str, unqualified: &'a str) -> &'a str {
    // An unqualified CREATE has unknown search_path; a later qualified ALTER can identify it.
    if !tables.contains_key(key) && tables.contains_key(unqualified) {
        unqualified
    } else {
        key
    }
}
