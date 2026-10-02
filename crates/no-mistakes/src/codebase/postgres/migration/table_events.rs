use crate::codebase::postgres::idents::{
    ident_key, object_name_key, unwrap_expr, visit_function_args,
};
use crate::codebase::postgres::schema::{column_metadata, table_metadata};
use crate::codebase::postgres::types::SqlTableSchemaEvent;
use sqlparser::ast::{
    AlterTableOperation, ColumnDef, ColumnOption, Expr, FunctionArguments, ObjectType, Statement,
};

mod positions;
pub(super) use positions::Positions;

pub(super) fn record(
    statement: &Statement,
    positions: &mut Positions,
    out: &mut Vec<SqlTableSchemaEvent>,
) {
    match statement {
        Statement::CreateTable(table) => out.push(SqlTableSchemaEvent::Create {
            if_not_exists: table.if_not_exists,
            source_order: positions.take("CREATE", &object_name_key(&table.name)),
            table: object_name_key(&table.name),
            unqualified_table: super::relation(&table.name),
            columns: table
                .columns
                .iter()
                .zip(table_metadata(table).columns)
                .map(|(definition, original)| {
                    let mut column = event_column(definition);
                    column.is_primary_key = original.is_primary_key;
                    column
                })
                .collect(),
        }),
        Statement::AlterTable(alter) => {
            let source_order = positions.take("ALTER", &object_name_key(&alter.name));
            for operation in &alter.operations {
                if let AlterTableOperation::AddColumn {
                    column_def,
                    if_not_exists,
                    ..
                } = operation
                {
                    out.push(SqlTableSchemaEvent::AddColumn {
                        if_not_exists: *if_not_exists,
                        source_order: source_order.clone(),
                        table: object_name_key(&alter.name),
                        unqualified_table: super::relation(&alter.name),
                        column: event_column(column_def),
                    });
                }
            }
        }
        Statement::Drop {
            object_type: ObjectType::Table,
            names,
            ..
        } => {
            for name in names {
                out.push(SqlTableSchemaEvent::Drop {
                    source_order: positions.take("DROP", &object_name_key(name)),
                    table: object_name_key(name),
                    unqualified_table: super::relation(name),
                });
            }
        }
        _ => {}
    }
}

fn event_column(column: &ColumnDef) -> crate::codebase::postgres::SqlColumnMetadata {
    let mut metadata = column_metadata(column);
    metadata.name = ident_key(&column.name);
    for option in &column.options {
        if let ColumnOption::Generated {
            generation_expr: Some(expr),
            ..
        } = &option.option
        {
            if let Expr::Function(function) = unwrap_expr(expr) {
                if let FunctionArguments::List(list) = &function.args {
                    metadata.generated_function_arg_columns.clear();
                    visit_function_args(&list.args, &mut |expr| {
                        let name = match unwrap_expr(expr) {
                            Expr::Identifier(name) => Some(ident_key(name)),
                            Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
                            _ => None,
                        };
                        metadata.generated_function_arg_columns.extend(name);
                    });
                }
            }
        }
    }
    metadata
}

#[cfg(test)]
mod tests;
