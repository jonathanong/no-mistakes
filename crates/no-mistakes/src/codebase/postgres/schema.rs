use super::parse::parse_postgres_sql_lenient;
use super::types::{SqlColumnMetadata, SqlCreateTableMetadata};
use sqlparser::ast::{
    ColumnOption, DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, IndexColumn,
    ObjectName, ObjectNamePart, Statement, TableConstraint,
};

/// Parse `sql` and return one metadata record per `CREATE TABLE`.
///
/// Unparseable statements are skipped so mixed migration files still yield
/// tables. `DO $$` bodies are peeled for schema DDL; other rejected SQL is
/// skipped rather than failing the file.
pub fn extract_create_table_metadata(sql: &str) -> Vec<SqlCreateTableMetadata> {
    parse_postgres_sql_lenient(sql)
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::CreateTable(table) => Some(table_metadata(&table)),
            _ => None,
        })
        .collect()
}

fn table_metadata(table: &sqlparser::ast::CreateTable) -> SqlCreateTableMetadata {
    let pk_columns = table_primary_key_columns(&table.constraints);
    let columns = table
        .columns
        .iter()
        .map(|column| {
            let mut facts = column_metadata(column);
            if pk_columns
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&facts.name))
            {
                facts.is_primary_key = true;
                push_constraint(&mut facts.constraints, "CONSTR_PRIMARY");
            }
            facts
        })
        .collect();
    SqlCreateTableMetadata {
        table_name: relation_name(&table.name),
        columns,
    }
}

fn column_metadata(column: &sqlparser::ast::ColumnDef) -> SqlColumnMetadata {
    let mut facts = SqlColumnMetadata {
        name: column.name.value.clone(),
        type_name: column_type_name(&column.data_type),
        constraints: Vec::new(),
        is_primary_key: false,
        is_generated: false,
        generated_expression: None,
        generated_function: None,
        generated_function_arg_columns: Vec::new(),
    };
    for option in &column.options {
        apply_column_option(&mut facts, &option.option);
    }
    facts
}

fn apply_column_option(facts: &mut SqlColumnMetadata, option: &ColumnOption) {
    match option {
        ColumnOption::Null => push_constraint(&mut facts.constraints, "CONSTR_NULL"),
        ColumnOption::NotNull => push_constraint(&mut facts.constraints, "CONSTR_NOTNULL"),
        ColumnOption::Default(_) => push_constraint(&mut facts.constraints, "CONSTR_DEFAULT"),
        ColumnOption::Unique(_) => push_constraint(&mut facts.constraints, "CONSTR_UNIQUE"),
        ColumnOption::ForeignKey(_) => push_constraint(&mut facts.constraints, "CONSTR_FOREIGN"),
        ColumnOption::Check(_) => push_constraint(&mut facts.constraints, "CONSTR_CHECK"),
        ColumnOption::PrimaryKey(_) => {
            facts.is_primary_key = true;
            push_constraint(&mut facts.constraints, "CONSTR_PRIMARY");
        }
        ColumnOption::Identity(_) => push_constraint(&mut facts.constraints, "CONSTR_IDENTITY"),
        ColumnOption::Generated {
            generation_expr,
            sequence_options,
            ..
        } => {
            if let Some(expr) = generation_expr {
                facts.is_generated = true;
                facts.generated_expression = Some(expr.to_string());
                facts.generated_function = generated_function(expr);
                facts.generated_function_arg_columns = generated_function_arg_columns(expr);
                push_constraint(&mut facts.constraints, "CONSTR_GENERATED");
            } else if sequence_options.is_some() {
                push_constraint(&mut facts.constraints, "CONSTR_IDENTITY");
            } else {
                push_constraint(&mut facts.constraints, "CONSTR_GENERATED");
                facts.is_generated = true;
            }
        }
        _ => {}
    }
}

fn table_primary_key_columns(constraints: &[TableConstraint]) -> Vec<String> {
    constraints
        .iter()
        .filter_map(|constraint| match constraint {
            TableConstraint::PrimaryKey(primary_key) => Some(
                primary_key
                    .columns
                    .iter()
                    .filter_map(index_column_name)
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect()
}

fn column_type_name(data_type: &DataType) -> Option<String> {
    match data_type {
        DataType::Unspecified => None,
        other => {
            let text = other.to_string();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
    }
}

fn push_constraint(constraints: &mut Vec<String>, token: &str) {
    if !constraints.iter().any(|existing| existing == token) {
        constraints.push(token.to_string());
    }
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

/// Function name of a generated-column expression, if the root is a call.
pub fn generated_function(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Function(function) => {
            let name = relation_name(&function.name);
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        }
        _ => None,
    }
}

/// Identifier arguments of a generated-column function, lowercased.
pub fn generated_function_arg_columns(expr: &Expr) -> Vec<String> {
    let Expr::Function(function) = unwrap_expr(expr) else {
        return Vec::new();
    };
    let FunctionArguments::List(list) = &function.args else {
        return Vec::new();
    };
    list.args
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
            expr_column_name(expr).map(|name| name.to_ascii_lowercase())
        })
        .collect()
}

/// Last identifier in an index column expression.
pub fn index_column_name(column: &IndexColumn) -> Option<String> {
    expr_column_name(&column.column.expr)
}

/// Relation name: the last identifier, not a schema qualifier.
pub fn relation_name(name: &ObjectName) -> String {
    name.0
        .iter()
        .rev()
        .find_map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(ident.value.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn expr_column_name(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident.value.clone()),
        Expr::CompoundIdentifier(parts) => parts.last().map(|ident| ident.value.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
