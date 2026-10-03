use crate::codebase::postgres::{SqlSchemaFileFacts, SqlTableSchemaEvent};
use std::collections::BTreeMap;
use std::path::Path;

mod ops;
use ops::{add, create, existing_key};

pub(crate) struct LiveColumn<'a> {
    pub(super) name: &'a str,
    pub(super) generated: bool,
}

pub(crate) struct LiveTable<'a> {
    pub(super) name: &'a str,
    pub(super) columns: Vec<LiveColumn<'a>>,
    pub(super) complete: bool,
    pub(super) temporary: bool,
}

pub(crate) type LiveTables<'a> = BTreeMap<String, LiveTable<'a>>;

pub(crate) fn live_tables<'a>(schema: &[&'a SqlSchemaFileFacts]) -> LiveTables<'a> {
    replay(schema, None)
}

/// Catalog state when `line` of `path` executes: earlier files plus that file's
/// earlier statements. Used for migration writes that precede later DDL.
pub(crate) fn tables_before<'a>(
    schema: &[&'a SqlSchemaFileFacts],
    path: &Path,
    line: usize,
) -> LiveTables<'a> {
    replay(schema, Some((path, line)))
}

fn replay<'a>(schema: &[&'a SqlSchemaFileFacts], cutoff: Option<(&Path, usize)>) -> LiveTables<'a> {
    let mut tables = BTreeMap::new();
    let mut ordered: Vec<_> = schema.iter().collect();
    ordered.sort_by(|left, right| {
        crate::codebase::postgres::cmp_sql_rel(
            &left.path.to_string_lossy().replace('\\', "/"),
            &right.path.to_string_lossy().replace('\\', "/"),
        )
    });
    for file in ordered {
        let target = cutoff.filter(|(path, _)| file.path == *path);
        apply_file(&mut tables, file, target.map(|(_, line)| line));
        if target.is_some() {
            break;
        }
    }
    tables
}

fn apply_file<'a>(tables: &mut LiveTables<'a>, file: &'a SqlSchemaFileFacts, limit: Option<usize>) {
    if !file.table_events_collected {
        // Public Rust callers can supply the original facts without ordered events.
        for table in &file.tables {
            create(
                tables,
                &table.table_name,
                &table.table_name,
                &table.columns,
                false,
                false,
                true,
            );
        }
        for column in &file.add_columns {
            let (key, table, name) = (
                &column.table_name,
                &column.unqualified_table_name,
                &column.column_name,
            );
            add(tables, key, table, name, column.is_generated, false, false);
        }
        return;
    }
    for event in &file.table_events {
        if limit.is_some_and(|line| event.line() >= line) {
            continue;
        }
        apply_event(tables, event);
    }
}

fn apply_event<'a>(tables: &mut LiveTables<'a>, event: &'a SqlTableSchemaEvent) {
    match event {
        SqlTableSchemaEvent::Create {
            relation_key,
            unqualified_table,
            columns,
            temporary,
            if_not_exists,
            columns_complete,
            ..
        } => create(
            tables,
            relation_key,
            unqualified_table,
            columns,
            *if_not_exists,
            *temporary,
            *columns_complete,
        ),
        SqlTableSchemaEvent::AddColumn {
            relation_key,
            unqualified_table,
            column,
            if_not_exists,
            table_if_exists,
            ..
        } => add(
            tables,
            relation_key,
            unqualified_table,
            &column.name,
            column.is_generated,
            *if_not_exists,
            *table_if_exists,
        ),
        SqlTableSchemaEvent::Drop {
            relation_key,
            unqualified_table,
            ..
        } => {
            if let Some(key) = existing_key(tables, relation_key, unqualified_table) {
                tables.remove(&key);
            }
        }
    }
}
