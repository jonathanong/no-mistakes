use super::{leading_index_column, qualified_relation};
use crate::codebase::postgres::types::{
    SqlCreateIndexMetadata, SqlDropIndexMetadata, SqlIndexParam,
};
use sqlparser::ast::{ColumnOption, IndexColumn, IndexType, ObjectName, TableConstraint};

pub(super) fn from_create_index(
    sql: &str,
    occurrence: usize,
    index: &sqlparser::ast::CreateIndex,
) -> SqlCreateIndexMetadata {
    let name = index.name.as_ref().map(qualified_relation);
    let columns = index_params(&index.columns);
    let leading_column = columns.first().and_then(|column| column.name.clone());
    SqlCreateIndexMetadata {
        table_name: qualified_relation(&index.table_name),
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
        not_null_predicate_column: index
            .predicate
            .as_ref()
            .and_then(super::predicate::not_null_predicate_column),
        predicate_key: index
            .predicate
            .as_ref()
            .map(super::predicate::predicate_key),
        line: super::lines::nth_create_index_line(sql, occurrence),
    }
}

pub(super) fn from_drop_index(
    sql: &str,
    occurrence: usize,
    names: &[ObjectName],
) -> Vec<SqlDropIndexMetadata> {
    drop_names(names, super::lines::nth_drop_index_line(sql, occurrence))
}

pub(super) fn from_drop_table(
    sql: &str,
    occurrence: usize,
    names: &[ObjectName],
) -> Vec<SqlDropIndexMetadata> {
    drop_names(names, super::lines::nth_drop_table_line(sql, occurrence))
}

fn drop_names(names: &[ObjectName], line: usize) -> Vec<SqlDropIndexMetadata> {
    names
        .iter()
        .map(|name| SqlDropIndexMetadata {
            name: qualified_relation(name),
            line,
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
        opclass: column.operator_class.as_ref().map(qualified_relation),
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

fn access_method(using: Option<&IndexType>) -> String {
    match using {
        None | Some(IndexType::BTree) => "btree".to_string(),
        Some(IndexType::Hash) => "hash".to_string(),
        Some(other) => other.to_string().to_ascii_lowercase(),
    }
}
