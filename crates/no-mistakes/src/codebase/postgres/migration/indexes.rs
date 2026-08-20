use super::{leading_index_column, line_containing, relation};
use crate::codebase::postgres::types::{
    SqlCreateIndexMetadata, SqlDropIndexMetadata, SqlIndexParam,
};
use sqlparser::ast::{ColumnOption, Expr, IndexColumn, IndexType, ObjectName, TableConstraint};

pub(super) fn from_create_index(
    sql: &str,
    index: &sqlparser::ast::CreateIndex,
) -> SqlCreateIndexMetadata {
    let name = index.name.as_ref().map(relation);
    let columns = index_params(&index.columns);
    let leading_column = columns.first().and_then(|column| column.name.clone());
    SqlCreateIndexMetadata {
        table_name: relation(&index.table_name),
        name: name.clone(),
        leading_column: leading_column.clone(),
        columns,
        include_columns: index
            .include
            .iter()
            .map(|ident| ident.value.clone())
            .collect(),
        access_method: access_method(index.using.as_ref()),
        unique: index.unique,
        has_predicate: index.predicate.is_some(),
        not_null_predicate_column: index.predicate.as_ref().and_then(not_null_predicate_column),
        predicate_key: index.predicate.as_ref().map(predicate_key),
        line: create_index_line(sql, name.as_deref(), leading_column.as_deref()),
    }
}

pub(super) fn from_drop_index(sql: &str, names: &[ObjectName]) -> Vec<SqlDropIndexMetadata> {
    names
        .iter()
        .map(|name| {
            let name = relation(name);
            SqlDropIndexMetadata {
                name: name.clone(),
                line: line_containing(sql, &["drop", "index", &name]),
            }
        })
        .collect()
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
            indexes.push(covering_unique(
                table_name,
                vec![named_param(&column.name.value)],
            ));
        }
    }
    for constraint in &table.constraints {
        match constraint {
            TableConstraint::Unique(unique) => {
                indexes.push(covering_from_index_columns(table_name, &unique.columns));
            }
            TableConstraint::PrimaryKey(primary) => {
                indexes.push(covering_from_index_columns(table_name, &primary.columns));
            }
            _ => {}
        }
    }
    indexes
}

pub(super) fn covering_from_index_columns(
    table_name: &str,
    columns: &[IndexColumn],
) -> SqlCreateIndexMetadata {
    covering_unique(table_name, index_params(columns))
}

pub(super) fn covering_unique(
    table_name: &str,
    columns: Vec<SqlIndexParam>,
) -> SqlCreateIndexMetadata {
    SqlCreateIndexMetadata {
        table_name: table_name.to_string(),
        leading_column: columns.first().and_then(|column| column.name.clone()),
        columns,
        unique: true,
        ..Default::default()
    }
}

fn named_param(name: &str) -> SqlIndexParam {
    SqlIndexParam {
        name: Some(name.to_string()),
        ..Default::default()
    }
}

fn index_params(columns: &[IndexColumn]) -> Vec<SqlIndexParam> {
    columns.iter().map(index_param).collect()
}

fn index_param(column: &IndexColumn) -> SqlIndexParam {
    SqlIndexParam {
        name: leading_index_column(column),
        opclass: column.operator_class.as_ref().map(relation),
        ordering: match column.column.options.asc {
            Some(true) => Some("asc".to_string()),
            Some(false) => Some("desc".to_string()),
            None => None,
        },
        nulls_ordering: match column.column.options.nulls_first {
            Some(true) => Some("first".to_string()),
            Some(false) => Some("last".to_string()),
            None => None,
        },
    }
}

fn create_index_line(sql: &str, name: Option<&str>, leading: Option<&str>) -> usize {
    if let Some(name) = name {
        return line_containing(sql, &["create", "index", name]);
    }
    leading
        .map(|column| line_containing(sql, &["create", "index", column]))
        .unwrap_or(1)
}

fn access_method(using: Option<&IndexType>) -> String {
    match using {
        None | Some(IndexType::BTree) => "btree".to_string(),
        Some(IndexType::Hash) => "hash".to_string(),
        Some(other) => other.to_string().to_ascii_lowercase(),
    }
}

fn predicate_key(expr: &Expr) -> String {
    unwrap_expr(expr)
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
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
