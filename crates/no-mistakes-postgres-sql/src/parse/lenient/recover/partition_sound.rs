use sqlparser::ast::{AlterTableOperation, DetachPartitionMode, Expr, ForValues, Statement};

pub(super) fn has_pg_partition(statement: &Statement) -> bool {
    let Statement::AlterTable(table) = statement else {
        return false;
    };
    table.operations.iter().any(|operation| {
        matches!(
            operation,
            AlterTableOperation::AttachPartitionOf { .. }
                | AlterTableOperation::DetachPartitionOf { .. }
        )
    })
}

/// sqlparser 0.64 accepts some `ATTACH`/`DETACH PARTITION` forms PostgreSQL rejects.
pub(super) fn acceptable(statement: &Statement, allow_concurrent_detach: bool) -> bool {
    if !allow_concurrent_detach && concurrent_detach(statement) {
        return false;
    }
    structurally_sound(statement)
}

fn concurrent_detach(statement: &Statement) -> bool {
    let Statement::AlterTable(table) = statement else {
        return false;
    };
    table.operations.iter().any(|operation| {
        matches!(
            operation,
            AlterTableOperation::DetachPartitionOf {
                mode: Some(DetachPartitionMode::Concurrently),
                ..
            }
        )
    })
}

fn structurally_sound(statement: &Statement) -> bool {
    let Statement::AlterTable(table) = statement else {
        return true;
    };
    let partitions = table
        .operations
        .iter()
        .filter(|operation| {
            matches!(
                operation,
                AlterTableOperation::AttachPartitionOf { .. }
                    | AlterTableOperation::DetachPartitionOf { .. }
            )
        })
        .count();
    if partitions == 0 {
        return true;
    }
    if table.operations.len() != 1 {
        return false;
    }
    match &table.operations[0] {
        AlterTableOperation::AttachPartitionOf { for_values, .. } => sound_bounds(for_values),
        _ => true,
    }
}

fn sound_bounds(for_values: &ForValues) -> bool {
    match for_values {
        ForValues::Default => true,
        ForValues::In(values) => values.iter().all(|value| !sentinel(value)),
        ForValues::From { from, to } => from.len() == to.len() && !from.is_empty(),
        ForValues::With { modulus, remainder } => *modulus > 0 && remainder < modulus,
    }
}

fn sentinel(expr: &Expr) -> bool {
    let Expr::Identifier(ident) = expr else {
        return false;
    };
    ident.value.eq_ignore_ascii_case("minvalue") || ident.value.eq_ignore_ascii_case("maxvalue")
}
