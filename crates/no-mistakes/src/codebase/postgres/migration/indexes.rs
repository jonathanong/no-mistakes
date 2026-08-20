use super::{leading_index_column, relation};
use crate::codebase::postgres::types::SqlCreateIndexMetadata;
use sqlparser::ast::{ColumnOption, Expr, IndexType, TableConstraint};

pub(super) fn from_create_index(index: &sqlparser::ast::CreateIndex) -> SqlCreateIndexMetadata {
    SqlCreateIndexMetadata {
        table_name: relation(&index.table_name),
        leading_column: index.columns.first().and_then(leading_index_column),
        access_method: access_method(index.using.as_ref()),
        has_predicate: index.predicate.is_some(),
        not_null_predicate_column: index.predicate.as_ref().and_then(not_null_predicate_column),
    }
}

pub(super) fn covering_from_table(
    table_name: &str,
    table: &sqlparser::ast::CreateTable,
) -> Vec<SqlCreateIndexMetadata> {
    let mut indexes = Vec::new();
    for column in &table.columns {
        let unique_or_pk = column.options.iter().any(|option| {
            matches!(
                option.option,
                ColumnOption::Unique(_) | ColumnOption::PrimaryKey(_)
            )
        });
        if unique_or_pk {
            indexes.push(covering_unique(table_name, Some(column.name.value.clone())));
        }
    }
    for constraint in &table.constraints {
        match constraint {
            TableConstraint::Unique(unique) => indexes.push(covering_unique(
                table_name,
                unique.columns.first().and_then(leading_index_column),
            )),
            TableConstraint::PrimaryKey(primary) => indexes.push(covering_unique(
                table_name,
                primary.columns.first().and_then(leading_index_column),
            )),
            _ => {}
        }
    }
    indexes
}

pub(super) fn covering_unique(
    table_name: &str,
    leading_column: Option<String>,
) -> SqlCreateIndexMetadata {
    SqlCreateIndexMetadata {
        table_name: table_name.to_string(),
        leading_column,
        access_method: "btree".to_string(),
        has_predicate: false,
        not_null_predicate_column: None,
    }
}

fn access_method(using: Option<&IndexType>) -> String {
    match using {
        None | Some(IndexType::BTree) => "btree".to_string(),
        Some(IndexType::Hash) => "hash".to_string(),
        Some(other) => other.to_string().to_ascii_lowercase(),
    }
}

fn not_null_predicate_column(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::IsNotNull(inner) => expr_column_name(inner),
        _ => None,
    }
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

fn expr_column_name(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident.value.clone()),
        Expr::CompoundIdentifier(parts) => parts.last().map(|ident| ident.value.clone()),
        _ => None,
    }
}
