use super::names::{collect_named_or_positional, push_assignment_writes, table_object_name};
use super::width::query_value_width;
use super::{GeneratedColumnWrite, GeneratedTable, GeneratedTableColumns};
use sqlparser::ast::{Insert, OnConflictAction, OnInsert};

pub(super) fn collect_insert_writes(
    insert: &Insert,
    catalog: &GeneratedTableColumns,
    writes: &mut Vec<GeneratedColumnWrite>,
) {
    let Some(table) = table_object_name(&insert.table) else {
        return;
    };
    let Some(meta) = catalog.get(&table) else {
        return;
    };
    let width = insert
        .source
        .as_ref()
        .and_then(|query| query_value_width(query));
    collect_named_or_positional(&table, meta, &insert.columns, width, writes);
    if let Some(OnInsert::OnConflict(conflict)) = &insert.on {
        if let OnConflictAction::DoUpdate(update) = &conflict.action {
            push_assignment_writes(&table, &update.assignments, catalog, writes);
        }
    }
}

/// Columnless INSERT hits when a VALUES/SELECT width covers a generated slot.
pub fn positional_insert_hits(
    _table: &str,
    meta: &GeneratedTable,
    width: Option<usize>,
) -> Vec<GeneratedColumnWrite> {
    let Some(order) = &meta.column_order else {
        return Vec::new();
    };
    let Some(width) = width else {
        return Vec::new();
    };
    order
        .iter()
        .enumerate()
        .filter(|(index, column)| *index < width && meta.generated.contains(*column))
        .map(|(_, column)| GeneratedColumnWrite {
            table: meta.name.clone(),
            column: column.clone(),
        })
        .collect()
}
