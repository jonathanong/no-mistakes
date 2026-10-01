use super::collect_one;
use crate::codebase::postgres::schema::relation_name;
use crate::codebase::postgres::statements::lines::line_containing;
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use sqlparser::ast::{FromTable, Statement, TableFactor, TableObject};

pub(super) fn returning(sql: &str, statement: &Statement) -> Vec<SqlStarProjectionFact> {
    match statement {
        Statement::Insert(insert) => match &insert.table {
            TableObject::TableName(name) => emit(
                sql,
                &relation_name(name),
                insert
                    .table_alias
                    .as_ref()
                    .map(|alias| alias.alias.value.clone()),
                insert.returning.as_deref(),
            ),
            _ => Vec::new(),
        },
        Statement::Update(update) => {
            let Some((table, alias)) = target_factor(&update.table.relation) else {
                return Vec::new();
            };
            emit(sql, &table, alias, update.returning.as_deref())
        }
        Statement::Delete(delete) => {
            let Some(table) = delete_target(&delete.from) else {
                return Vec::new();
            };
            let Some((name, alias)) = target_factor(&table.relation) else {
                return Vec::new();
            };
            emit(sql, &name, alias, delete.returning.as_deref())
        }
        Statement::Explain { statement, .. } => returning(sql, statement),
        _ => Vec::new(),
    }
}

fn emit(
    sql: &str,
    table: &str,
    alias: Option<String>,
    items: Option<&[sqlparser::ast::SelectItem]>,
) -> Vec<SqlStarProjectionFact> {
    let Some(items) = items else {
        return Vec::new();
    };
    if table.is_empty() {
        return Vec::new();
    }
    let line = line_containing(sql, &[table]);
    collect_one(table, alias.as_deref(), items, line)
}

fn delete_target(from: &FromTable) -> Option<&sqlparser::ast::TableWithJoins> {
    match from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables.first(),
    }
}

fn target_factor(factor: &TableFactor) -> Option<(String, Option<String>)> {
    match factor {
        TableFactor::Table { name, alias, .. } => Some((
            relation_name(name),
            alias.as_ref().map(|alias| alias.name.value.clone()),
        )),
        _ => None,
    }
}
