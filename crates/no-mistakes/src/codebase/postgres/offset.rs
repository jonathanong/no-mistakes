mod sql_text;
#[cfg(test)]
mod tests;
mod walk;

use super::parse::{parse_postgres_sql, parse_postgres_sql_lenient, PostgresParseError};
use super::statements::walk_executed;
use sqlparser::ast::{
    CopySource, CreateTable, CreateView, Delete, DoUpdate, Insert, OnConflict, OnConflictAction,
    OnInsert, SelectItem, Statement, Update,
};
use walk::{expr_offsets, query_offsets, select_item_offsets, table_with_joins_offsets};

/// One `OFFSET` clause. `Zero` is the integer literal `0`, including `OFFSET 0 ROWS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetUse {
    Zero,
    Other,
}

/// Parse `sql` and return every `OFFSET` clause in source order.
pub fn sql_offset_uses(sql: &str) -> Result<Vec<OffsetUse>, PostgresParseError> {
    let statements = parse_postgres_sql(sql)?;
    let mut uses = Vec::new();
    for statement in &statements {
        statement_offsets(statement, &mut uses);
    }
    Ok(uses)
}

/// Parse `sql` and report whether any query uses an `OFFSET` clause.
pub fn sql_has_offset_clause(sql: &str) -> Result<bool, PostgresParseError> {
    Ok(!sql_offset_uses(sql)?.is_empty())
}

/// Offset uses in a `.sql` file, using the statement pass's lenient split.
///
/// Only top-level queries and `CREATE [MATERIALIZED] VIEW` queries are walked.
/// Each use is reported at that statement's start line.
pub fn sql_file_offset_uses(sql: &str) -> Vec<(usize, OffsetUse)> {
    let mut found = Vec::new();
    for (line, chunk) in sql_text::top_level_statements(sql) {
        let statements = parse_postgres_sql_lenient(&chunk);
        let mut executed = Vec::new();
        for statement in &statements {
            walk_executed(statement, &mut executed);
        }
        for statement in executed {
            let mut uses = Vec::new();
            match statement {
                Statement::Query(query) => query_offsets(query, &mut uses),
                Statement::CreateView(view) => query_offsets(view.query.as_ref(), &mut uses),
                _ => {}
            }
            for use_ in uses {
                found.push((line, use_));
            }
        }
    }
    found
}

pub(super) fn statement_offsets(statement: &Statement, out: &mut Vec<OffsetUse>) {
    match statement {
        Statement::Query(query) => query_offsets(query, out),
        Statement::Insert(Insert {
            source,
            returning,
            on,
            ..
        }) => {
            if let Some(query) = source.as_ref() {
                query_offsets(query, out);
            }
            returning_offsets(returning, out);
            insert_on_offsets(on, out);
        }
        Statement::Update(Update {
            assignments,
            selection,
            table,
            from,
            returning,
            ..
        }) => {
            for assignment in assignments {
                expr_offsets(&assignment.value, out);
            }
            if let Some(selection) = selection {
                expr_offsets(selection, out);
            }
            table_with_joins_offsets(table, out);
            if let Some(sqlparser::ast::UpdateTableFromKind::AfterSet(tables)) = from.as_ref() {
                for table in tables {
                    table_with_joins_offsets(table, out);
                }
            }
            returning_offsets(returning, out);
        }
        Statement::Delete(Delete {
            selection,
            using,
            returning,
            ..
        }) => {
            if let Some(selection) = selection {
                expr_offsets(selection, out);
            }
            if let Some(tables) = using {
                for table in tables {
                    table_with_joins_offsets(table, out);
                }
            }
            returning_offsets(returning, out);
        }
        Statement::CreateTable(CreateTable { query, .. }) => {
            if let Some(query) = query.as_deref() {
                query_offsets(query, out);
            }
        }
        Statement::CreateView(CreateView { query, .. }) => query_offsets(query, out),
        Statement::Copy {
            source: CopySource::Query(query),
            ..
        } => query_offsets(query, out),
        Statement::Explain { statement, .. } => statement_offsets(statement, out),
        _ => {}
    }
}

fn returning_offsets(returning: &Option<Vec<SelectItem>>, out: &mut Vec<OffsetUse>) {
    if let Some(items) = returning {
        for item in items {
            select_item_offsets(item, out);
        }
    }
}

fn insert_on_offsets(on: &Option<OnInsert>, out: &mut Vec<OffsetUse>) {
    let Some(OnInsert::OnConflict(OnConflict {
        action:
            OnConflictAction::DoUpdate(DoUpdate {
                assignments,
                selection,
                ..
            }),
        ..
    })) = on
    else {
        return;
    };
    for assignment in assignments {
        expr_offsets(&assignment.value, out);
    }
    if let Some(selection) = selection {
        expr_offsets(selection, out);
    }
}
