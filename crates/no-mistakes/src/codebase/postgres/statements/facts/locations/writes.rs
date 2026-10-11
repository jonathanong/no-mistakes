use super::at;
use crate::codebase::postgres::dml::writes::names as write_names;
use sqlparser::ast::{
    Assignment, AssignmentTarget, MergeAction, MergeUpdateKind, OnConflictAction, OnInsert,
    Spanned, Statement, TableObject,
};

pub(super) struct Write {
    pub table: String,
    pub line: usize,
    pub at: (usize, usize),
    pub columns: Vec<(String, (usize, usize))>,
}

pub(super) fn collect(statement: &Statement) -> Vec<Write> {
    let origin = at(statement.span());
    let mut out = Vec::new();
    let mut push = |table: String, position, columns| {
        out.push(Write {
            table,
            line: origin.0,
            at: position,
            columns,
        })
    };
    match statement {
        Statement::Insert(insert) => {
            if let TableObject::TableName(name) = &insert.table {
                let table = crate::codebase::postgres::idents::object_name_identity(name);
                push(table.clone(), origin, names(&insert.columns));
                if let Some(OnInsert::OnConflict(conflict)) = &insert.on {
                    if let OnConflictAction::DoUpdate(update) = &conflict.action {
                        let columns = assignments(&update.assignments);
                        let position = columns.first().map_or(origin, |(_, position)| *position);
                        push(table, position, columns);
                    }
                }
            }
        }
        Statement::Update(update) => {
            if let Some(table) = write_names::table_with_joins_name(&update.table) {
                push(table, origin, assignments(&update.assignments));
            }
        }
        Statement::Merge(merge) => {
            if let Some(table) = write_names::table_factor_name(&merge.table) {
                for clause in &merge.clauses {
                    match &clause.action {
                        MergeAction::Insert(insert) => push(
                            table.clone(),
                            at(insert.insert_token.0.span),
                            names(&insert.columns),
                        ),
                        MergeAction::Update(update) => {
                            let columns = match &update.kind {
                                MergeUpdateKind::Set(values) => assignments(values),
                                MergeUpdateKind::Wildcard => Vec::new(),
                            };
                            push(table.clone(), at(update.update_token.0.span), columns);
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
    out
}

fn names(names: &[sqlparser::ast::ObjectName]) -> Vec<(String, (usize, usize))> {
    names
        .iter()
        .map(|name| {
            (
                crate::codebase::postgres::schema::relation_name(name),
                at(name.span()),
            )
        })
        .collect()
}

fn assignments(values: &[Assignment]) -> Vec<(String, (usize, usize))> {
    values
        .iter()
        .flat_map(|assignment| match &assignment.target {
            AssignmentTarget::ColumnName(name) => names(std::slice::from_ref(name)),
            AssignmentTarget::Tuple(values) => names(values),
        })
        .collect()
}
