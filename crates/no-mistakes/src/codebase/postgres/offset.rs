#[cfg(test)]
mod tests;
mod walk;

use super::parse::{parse_postgres_sql, PostgresParseError};
use sqlparser::ast::{
    CopySource, CreateTable, CreateView, Delete, DoUpdate, Insert, OnConflict, OnConflictAction,
    OnInsert, SelectItem, Statement, Update,
};
use walk::{expr_has_offset_query, query_has_offset, table_with_joins_has_offset};

/// Parse `sql` and report whether any query uses an `OFFSET` clause.
pub fn sql_has_offset_clause(sql: &str) -> Result<bool, PostgresParseError> {
    let statements = parse_postgres_sql(sql)?;
    Ok(statements.iter().any(statement_has_offset))
}

pub(super) fn statement_has_offset(statement: &Statement) -> bool {
    match statement {
        Statement::Query(query) => query_has_offset(query),
        Statement::Insert(Insert {
            source,
            returning,
            on,
            ..
        }) => {
            source.as_ref().is_some_and(|query| query_has_offset(query))
                || returning_has_offset(returning)
                || insert_on_has_offset(on)
        }
        Statement::Update(Update {
            assignments,
            selection,
            table,
            from,
            returning,
            ..
        }) => {
            assignments
                .iter()
                .any(|assignment| expr_has_offset_query(&assignment.value))
                || selection.as_ref().is_some_and(expr_has_offset_query)
                || table_with_joins_has_offset(table)
                || matches!(
                    from.as_ref(),
                    Some(sqlparser::ast::UpdateTableFromKind::AfterSet(tables))
                        if tables.iter().any(table_with_joins_has_offset)
                )
                || returning_has_offset(returning)
        }
        Statement::Delete(Delete {
            selection,
            using,
            returning,
            ..
        }) => {
            selection.as_ref().is_some_and(expr_has_offset_query)
                || using
                    .as_ref()
                    .is_some_and(|tables| tables.iter().any(table_with_joins_has_offset))
                || returning_has_offset(returning)
        }
        Statement::CreateTable(CreateTable { query, .. }) => {
            query.as_deref().is_some_and(query_has_offset)
        }
        Statement::CreateView(CreateView { query, .. }) => query_has_offset(query),
        Statement::Copy {
            source: CopySource::Query(query),
            ..
        } => query_has_offset(query),
        Statement::Explain {
            analyze: true,
            statement,
            ..
        } => statement_has_offset(statement),
        _ => false,
    }
}

fn returning_has_offset(returning: &Option<Vec<SelectItem>>) -> bool {
    returning
        .as_ref()
        .is_some_and(|items| items.iter().any(walk::select_item_has_offset))
}

fn insert_on_has_offset(on: &Option<OnInsert>) -> bool {
    matches!(
        on,
        Some(OnInsert::OnConflict(OnConflict {
            action: OnConflictAction::DoUpdate(DoUpdate {
                assignments,
                selection,
                ..
            }),
            ..
        })) if assignments
            .iter()
            .any(|assignment| expr_has_offset_query(&assignment.value))
            || selection.as_ref().is_some_and(expr_has_offset_query)
    )
}
