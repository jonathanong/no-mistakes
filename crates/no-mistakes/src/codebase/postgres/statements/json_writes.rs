//! JSON key syntax projected from the request's already parsed SQL AST.
mod keys;
mod rows;
use crate::codebase::postgres::dml::writes::names;
use crate::codebase::postgres::statements::{SqlJsonWriteFact, SqlWriteColumns};
use sqlparser::ast::{Assignment, AssignmentTarget, Expr, Statement, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn collect(statement: &Statement, out: &mut Vec<SqlJsonWriteFact>) {
    super::facts::locations::work::json_key_walk();
    struct Writes<'a>(&'a mut Vec<SqlJsonWriteFact>);
    impl Visitor for Writes<'_> {
        type Break = ();
        fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
            match statement {
                Statement::Insert(insert) => {
                    if let Some(table) = names::table_object_name(&insert.table) {
                        rows::insert(&table, insert, self.0);
                        assignments(&table, &insert.assignments, self.0);
                        if let Some(sqlparser::ast::OnInsert::OnConflict(conflict)) = &insert.on {
                            if let sqlparser::ast::OnConflictAction::DoUpdate(update) =
                                &conflict.action
                            {
                                assignments(&table, &update.assignments, self.0);
                            }
                        }
                    }
                }
                Statement::Update(update) => {
                    if let Some(table) = names::table_with_joins_name(&update.table) {
                        assignments(&table, &update.assignments, self.0);
                    }
                }
                _ => {}
            }
            ControlFlow::Continue(())
        }
    }
    let _: ControlFlow<()> = statement.visit(&mut Writes(out));
}

fn assignments(table: &str, values: &[Assignment], out: &mut Vec<SqlJsonWriteFact>) {
    let columns: Vec<_> = values
        .iter()
        .flat_map(|value| column_names(&value.target))
        .collect();
    let written = SqlWriteColumns::Named(columns);
    for value in values {
        let columns = column_names(&value.target);
        if let AssignmentTarget::Tuple(_) = &value.target {
            if let Expr::Tuple(exprs) = crate::codebase::postgres::idents::unwrap_expr(&value.value)
            {
                for (index, expr) in exprs.iter().enumerate() {
                    push(
                        table,
                        columns.get(index).cloned(),
                        Some(index),
                        &written,
                        expr,
                        out,
                    );
                }
            }
        } else {
            push(
                table,
                columns.first().cloned(),
                Some(0),
                &written,
                &value.value,
                out,
            );
        }
    }
}

fn column_names(target: &AssignmentTarget) -> Vec<String> {
    match target {
        AssignmentTarget::ColumnName(name) => vec![column_name(name)],
        AssignmentTarget::Tuple(names) => names.iter().map(column_name).collect(),
    }
}

fn column_name(name: &sqlparser::ast::ObjectName) -> String {
    crate::codebase::postgres::idents::object_name_ident(name)
        .map(crate::codebase::postgres::idents::ident_key)
        .unwrap_or_default()
}

fn push(
    table: &str,
    column: Option<String>,
    column_index: Option<usize>,
    written_columns: &SqlWriteColumns,
    expr: &Expr,
    out: &mut Vec<SqlJsonWriteFact>,
) {
    for (key, span) in keys::collect(expr) {
        out.push(SqlJsonWriteFact {
            table: table.to_string(),
            column: column.clone(),
            column_index,
            written_columns: written_columns.clone(),
            key,
            line: span.start.line.max(1) as usize,
            column_position: span.start.column.max(1) as usize,
        });
    }
}
