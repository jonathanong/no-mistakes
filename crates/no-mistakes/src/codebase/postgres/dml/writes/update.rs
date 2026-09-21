use super::names::{
    collect_named_or_positional, push_all_generated, push_assignment_writes, table_factor_name,
    table_with_joins_name,
};
use super::{GeneratedColumnWrite, GeneratedTable, GeneratedTableColumns};
use sqlparser::ast::{Merge, MergeAction, MergeInsertKind, MergeUpdateKind, Update};

pub(super) fn collect_update_writes(
    update: &Update,
    catalog: &GeneratedTableColumns,
    writes: &mut Vec<GeneratedColumnWrite>,
) {
    let Some(table) = table_with_joins_name(&update.table) else {
        return;
    };
    push_assignment_writes(&table, &update.assignments, catalog, writes);
}

pub(super) fn collect_merge_writes(
    merge: &Merge,
    catalog: &GeneratedTableColumns,
    writes: &mut Vec<GeneratedColumnWrite>,
) {
    let Some(table) = table_factor_name(&merge.table) else {
        return;
    };
    let Some(meta) = catalog.get(&table) else {
        return;
    };
    for clause in &merge.clauses {
        match &clause.action {
            MergeAction::Update(update) => match &update.kind {
                MergeUpdateKind::Set(assignments) => {
                    push_assignment_writes(&table, assignments, catalog, writes);
                }
                MergeUpdateKind::Wildcard => push_all_generated(meta, writes),
            },
            MergeAction::Insert(insert) => {
                collect_named_or_positional(
                    &table,
                    meta,
                    &insert.columns,
                    merge_insert_width(&insert.kind, meta),
                    writes,
                );
            }
            MergeAction::Delete { .. } | MergeAction::DoNothing { .. } => {}
        }
    }
}

pub(super) fn merge_insert_width(kind: &MergeInsertKind, meta: &GeneratedTable) -> Option<usize> {
    match kind {
        MergeInsertKind::Values(values) => values.rows.iter().map(|row| row.len()).max(),
        MergeInsertKind::Row | MergeInsertKind::Wildcard => {
            meta.column_order.as_ref().map(Vec::len)
        }
    }
}
