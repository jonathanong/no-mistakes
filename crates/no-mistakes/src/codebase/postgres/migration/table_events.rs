use crate::codebase::postgres::idents::{ident_key, object_name_key, unwrap_expr};
use crate::codebase::postgres::schema::{column_metadata, table_metadata};
use crate::codebase::postgres::types::SqlTableSchemaEvent;
use sqlparser::ast::{
    AlterTableOperation, ColumnDef, ColumnOption, Expr, FunctionArg, FunctionArgExpr,
    FunctionArguments, ObjectType, Statement,
};

pub(super) fn record(statement: &Statement, out: &mut Vec<SqlTableSchemaEvent>) {
    match statement {
        Statement::CreateTable(table) => out.push(SqlTableSchemaEvent::Create {
            table: object_name_key(&table.name),
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
            for operation in &alter.operations {
                if let AlterTableOperation::AddColumn { column_def, .. } = operation {
                    out.push(SqlTableSchemaEvent::AddColumn {
                        table: object_name_key(&alter.name),
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
                    table: object_name_key(name),
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
                    metadata.generated_function_arg_columns = list
                        .args
                        .iter()
                        .filter_map(|arg| {
                            let expr = match arg {
                                FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
                                | FunctionArg::Named {
                                    arg: FunctionArgExpr::Expr(expr),
                                    ..
                                }
                                | FunctionArg::ExprNamed {
                                    arg: FunctionArgExpr::Expr(expr),
                                    ..
                                } => expr,
                                _ => return None,
                            };
                            match unwrap_expr(expr) {
                                Expr::Identifier(name) => Some(ident_key(name)),
                                Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
                                _ => None,
                            }
                        })
                        .collect();
                }
            }
        }
    }
    metadata
}
