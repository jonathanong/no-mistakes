use super::super::super::schema::relation_name;
use super::{GeneratedColumnWrite, GeneratedTable, GeneratedTableColumns};
use sqlparser::ast::{
    Assignment, AssignmentTarget, ObjectName, TableFactor, TableObject, TableWithJoins,
};

pub(super) fn push_assignment_writes(
    table: &str,
    assignments: &[Assignment],
    catalog: &GeneratedTableColumns,
    writes: &mut Vec<GeneratedColumnWrite>,
) {
    let Some(meta) = catalog.get(table) else {
        return;
    };
    for assignment in assignments {
        for name in assignment_column_names(&assignment.target) {
            push_if_generated(meta, &name, writes);
        }
    }
}

pub(super) fn collect_named_or_positional(
    table: &str,
    meta: &GeneratedTable,
    columns: &[ObjectName],
    positional_width: Option<usize>,
    writes: &mut Vec<GeneratedColumnWrite>,
) {
    if columns.is_empty() {
        writes.extend(super::positional_insert_hits(table, meta, positional_width));
        return;
    }
    for column in columns {
        push_if_generated(meta, &relation_name(column), writes);
    }
}

pub(super) fn push_if_generated(
    meta: &GeneratedTable,
    column: &str,
    writes: &mut Vec<GeneratedColumnWrite>,
) {
    if meta.generated.contains(&column.to_ascii_lowercase()) {
        writes.push(GeneratedColumnWrite {
            table: meta.name.clone(),
            column: column.to_string(),
        });
    }
}

fn assignment_column_names(target: &AssignmentTarget) -> Vec<String> {
    match target {
        AssignmentTarget::ColumnName(name) => vec![relation_name(name)],
        AssignmentTarget::Tuple(names) => names.iter().map(relation_name).collect(),
    }
}

pub(super) fn table_object_name(table: &TableObject) -> Option<String> {
    match table {
        TableObject::TableName(name) => Some(relation_name(name)),
        _ => None,
    }
}

pub(super) fn table_with_joins_name(table: &TableWithJoins) -> Option<String> {
    table_factor_name(&table.relation)
}

pub(super) fn table_factor_name(table: &TableFactor) -> Option<String> {
    match table {
        TableFactor::Table { name, .. } => Some(relation_name(name)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
