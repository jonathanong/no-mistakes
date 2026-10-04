use super::{
    expressions::{expression, identifier, name},
    locations::Locations,
    type_facts::data_type,
    types::*,
};
use sqlparser::ast::{ColumnDef, ColumnOption, Expr, IndexColumn, TableConstraint};

pub(super) fn column(value: &ColumnDef, locations: &Locations<'_>) -> PostgresSqlColumn {
    let mut result = PostgresSqlColumn {
        name: identifier(&value.name),
        data_type: data_type(&value.data_type, locations),
        nullable: true,
        default: None,
        generated: None,
        identity: None,
        constraints: Vec::new(),
        span: locations.span(value.name.span),
    };
    for option in &value.options {
        let constraint = match &option.option {
            ColumnOption::NotNull => {
                result.nullable = false;
                None
            }
            ColumnOption::Null => {
                result.nullable = true;
                None
            }
            ColumnOption::Default(expr) => {
                result.default = Some(expression(expr, locations));
                None
            }
            ColumnOption::PrimaryKey(value) => {
                result.nullable = false;
                Some(TableConstraint::PrimaryKey(value.clone()))
            }
            ColumnOption::Unique(value) => Some(TableConstraint::Unique(value.clone())),
            ColumnOption::ForeignKey(value) => Some(TableConstraint::ForeignKey(value.clone())),
            ColumnOption::Check(value) => Some(TableConstraint::Check(value.clone())),
            ColumnOption::Generated {
                generated_as,
                generation_expr,
                generation_expr_mode,
                sequence_options,
                ..
            } => {
                if let Some(expr) = generation_expr {
                    result.generated = Some(PostgresSqlGeneratedColumn {
                        expression: expression(expr, locations),
                        storage: generation_expr_mode.as_ref().map(|mode| match mode {
                            sqlparser::ast::GeneratedExpressionMode::Stored => "STORED".into(),
                            sqlparser::ast::GeneratedExpressionMode::Virtual => "VIRTUAL".into(),
                        }),
                    });
                } else {
                    result.identity = Some(PostgresSqlIdentityColumn {
                        mode: format!("{generated_as:?}"),
                        options: sequence_options
                            .iter()
                            .flatten()
                            .map(ToString::to_string)
                            .collect(),
                    });
                }
                None
            }
            _ => None,
        };
        if let Some(value) = constraint {
            let mut fact = table_constraint(&value, locations);
            fact.name = option.name.as_ref().map(identifier).or(fact.name);
            // Inline constraints own this column; their parser payload has no key list.
            fact.columns = vec![result.name.clone()];
            result.constraints.push(fact);
        }
    }
    result
}

fn keys(columns: &[IndexColumn]) -> Vec<PostgresSqlIdentifier> {
    columns
        .iter()
        .filter_map(|column| match &column.column.expr {
            Expr::Identifier(ident) => Some(identifier(ident)),
            _ => None,
        })
        .collect()
}

pub(super) fn table_constraint(
    value: &TableConstraint,
    locations: &Locations<'_>,
) -> PostgresSqlConstraint {
    let mut result = PostgresSqlConstraint {
        kind: PostgresSqlConstraintKind::Other,
        name: None,
        columns: Vec::new(),
        expression: None,
        referenced_table: None,
        referenced_columns: Vec::new(),
        on_delete: None,
        on_update: None,
        characteristics: None,
        sql: value.to_string(),
    };
    match value {
        TableConstraint::PrimaryKey(value) => {
            result.kind = PostgresSqlConstraintKind::PrimaryKey;
            result.name = value.name.as_ref().map(identifier);
            result.columns = keys(&value.columns);
            result.characteristics = value.characteristics.as_ref().map(ToString::to_string);
        }
        TableConstraint::Unique(value) => {
            result.kind = PostgresSqlConstraintKind::Unique;
            result.name = value.name.as_ref().map(identifier);
            result.columns = keys(&value.columns);
            result.characteristics = value.characteristics.as_ref().map(ToString::to_string);
        }
        TableConstraint::ForeignKey(value) => {
            result.kind = PostgresSqlConstraintKind::ForeignKey;
            result.name = value.name.as_ref().map(identifier);
            result.columns = value.columns.iter().map(identifier).collect();
            result.referenced_table = Some(name(&value.foreign_table));
            result.referenced_columns = value.referred_columns.iter().map(identifier).collect();
            result.on_delete = value.on_delete.as_ref().map(ToString::to_string);
            result.on_update = value.on_update.as_ref().map(ToString::to_string);
            result.characteristics = value.characteristics.as_ref().map(ToString::to_string);
        }
        TableConstraint::Check(value) => {
            result.kind = PostgresSqlConstraintKind::Check;
            result.name = value.name.as_ref().map(identifier);
            result.expression = Some(expression(&value.expr, locations));
        }
        _ => {}
    }
    result
}
