use super::{
    columns::{column, table_constraint},
    expressions::{expression, identifier},
    locations::Locations,
    type_facts::data_type,
    types::*,
};
use sqlparser::ast::{AlterColumnOperation, AlterTableOperation};
pub(super) fn alter(
    value: &AlterTableOperation,
    locations: &Locations<'_>,
    span: Option<super::PostgresSqlSpan>,
    option_spans: Vec<Option<super::PostgresSqlSpan>>,
) -> PostgresSqlAlterOperation {
    match value {
        AlterTableOperation::AddColumn { column_def, .. } => PostgresSqlAlterOperation::AddColumn {
            column: Box::new(column(column_def, locations, option_spans)),
        },
        AlterTableOperation::AddConstraint {
            constraint,
            not_valid,
        } => PostgresSqlAlterOperation::AddConstraint {
            constraint: Box::new(table_constraint(constraint, locations, span)),
            not_valid: *not_valid,
        },
        AlterTableOperation::ValidateConstraint { name } => {
            PostgresSqlAlterOperation::ValidateConstraint {
                name: identifier(name),
            }
        }
        AlterTableOperation::AlterColumn { column_name, op } => {
            let column = identifier(column_name);
            match op {
                AlterColumnOperation::SetDataType {
                    data_type: value,
                    using,
                    ..
                } => PostgresSqlAlterOperation::AlterColumnType {
                    column,
                    data_type: data_type(value, locations),
                    using: using.as_ref().map(|expr| expression(expr, locations)),
                },
                AlterColumnOperation::SetDefault { value } => {
                    PostgresSqlAlterOperation::SetDefault {
                        column,
                        expression: expression(value, locations),
                    }
                }
                AlterColumnOperation::DropDefault => {
                    PostgresSqlAlterOperation::DropDefault { column }
                }
                AlterColumnOperation::SetNotNull => {
                    PostgresSqlAlterOperation::SetNotNull { column }
                }
                AlterColumnOperation::DropNotNull => {
                    PostgresSqlAlterOperation::DropNotNull { column }
                }
                _ => PostgresSqlAlterOperation::Other {
                    sql: value.to_string(),
                },
            }
        }
        _ => PostgresSqlAlterOperation::Other {
            sql: value.to_string(),
        },
    }
}
