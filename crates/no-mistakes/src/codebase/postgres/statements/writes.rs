use crate::codebase::postgres::dml::writes::{names, width};
use crate::codebase::postgres::{SqlWriteColumns, SqlWriteFact};
use sqlparser::ast::Spanned;
use sqlparser::ast::{
    MergeAction, MergeInsertKind, MergeUpdateKind, OnConflictAction, OnInsert, Statement,
};

pub(super) fn collect(statement: &Statement, out: &mut Vec<SqlWriteFact>) {
    let line = statement.span().start.line.max(1) as usize;
    let column = statement.span().start.column.max(1) as usize;
    match statement {
        Statement::Insert(insert) => {
            let Some(table) = names::table_object_name(&insert.table) else {
                return;
            };
            let column_keys = (!insert.columns.is_empty())
                .then(|| insert.columns.iter().map(column_key).collect());
            let columns = if insert.columns.is_empty() {
                SqlWriteColumns::Positional(
                    insert.source.as_deref().and_then(width::query_value_width),
                )
            } else {
                SqlWriteColumns::Named(
                    insert
                        .columns
                        .iter()
                        .map(crate::codebase::postgres::schema::relation_name)
                        .collect(),
                )
            };
            out.push(SqlWriteFact {
                table: table.clone(),
                columns,
                line,
                column,
                column_keys,
            });
            if let Some(OnInsert::OnConflict(conflict)) = &insert.on {
                if let OnConflictAction::DoUpdate(update) = &conflict.action {
                    assignments(&table, &update.assignments, line, column, out);
                }
            }
        }
        Statement::Update(update) => {
            if let Some(table) = names::table_with_joins_name(&update.table) {
                assignments(&table, &update.assignments, line, column, out);
            }
        }
        Statement::Merge(merge) => {
            let Some(table) = names::table_factor_name(&merge.table) else {
                return;
            };
            for clause in &merge.clauses {
                match &clause.action {
                    MergeAction::Update(update) => match &update.kind {
                        MergeUpdateKind::Set(values) => {
                            assignments(&table, values, line, column, out)
                        }
                        MergeUpdateKind::Wildcard => out.push(SqlWriteFact {
                            table: table.clone(),
                            columns: SqlWriteColumns::All,
                            line,
                            column,
                            column_keys: None,
                        }),
                    },
                    MergeAction::Insert(insert) => {
                        let column_keys = (!insert.columns.is_empty())
                            .then(|| insert.columns.iter().map(column_key).collect());
                        let columns = if !insert.columns.is_empty() {
                            SqlWriteColumns::Named(
                                insert
                                    .columns
                                    .iter()
                                    .map(crate::codebase::postgres::schema::relation_name)
                                    .collect(),
                            )
                        } else {
                            match &insert.kind {
                                MergeInsertKind::Wildcard => SqlWriteColumns::All,
                                MergeInsertKind::Row => {
                                    SqlWriteColumns::Positional(Some(usize::MAX))
                                }
                                MergeInsertKind::Values(values) => SqlWriteColumns::Positional(
                                    values.rows.iter().map(|row| row.len()).max(),
                                ),
                            }
                        };
                        out.push(SqlWriteFact {
                            table: table.clone(),
                            columns,
                            line,
                            column,
                            column_keys,
                        });
                    }
                    MergeAction::Delete { .. } | MergeAction::DoNothing { .. } => {}
                }
            }
        }
        _ => {}
    }
}

fn assignments(
    table: &str,
    values: &[sqlparser::ast::Assignment],
    line: usize,
    column: usize,
    out: &mut Vec<SqlWriteFact>,
) {
    out.push(SqlWriteFact {
        table: table.into(),
        columns: SqlWriteColumns::Named(
            values
                .iter()
                .flat_map(|value| names::assignment_column_names(&value.target))
                .collect(),
        ),
        line,
        column,
        column_keys: Some(
            values
                .iter()
                .flat_map(|value| names::assignment_column_keys(&value.target))
                .collect(),
        ),
    });
}

fn column_key(name: &sqlparser::ast::ObjectName) -> String {
    crate::codebase::postgres::idents::object_name_ident(name)
        .map(crate::codebase::postgres::idents::ident_key)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
