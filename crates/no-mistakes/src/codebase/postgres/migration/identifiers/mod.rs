use super::relation;
use crate::codebase::postgres::types::SqlDeclaredIdentifier;
use sqlparser::ast::{
    AlterIndexOperation, AlterTableOperation, ColumnDef, ObjectName, RenameTableNameKind,
    Statement, TableConstraint,
};

mod locate;
mod procedures;

pub(super) fn procedure_names(
    sql: &str,
) -> Vec<crate::codebase::postgres::types::SqlDeclaredIdentifier> {
    procedures::procedure_names(sql)
}

pub(super) fn collect(
    sql: &str,
    statement: &Statement,
    from: &mut usize,
) -> Vec<SqlDeclaredIdentifier> {
    let Some(words) = opening_words(statement) else {
        return Vec::new();
    };
    let line = statement_line(sql, from, words);
    let mut names = Vec::new();
    push_statement(&mut names, statement, line);
    names
}

fn statement_line(sql: &str, from: &mut usize, words: &[&str]) -> usize {
    let rest = sql.get(*from..).unwrap_or("");
    let Some((start, end)) = locate::find_opening(rest, words) else {
        return 1;
    };
    let absolute = *from + start;
    *from += end;
    locate::line_number(sql, absolute)
}

fn opening_words(statement: &Statement) -> Option<&'static [&'static str]> {
    Some(match statement {
        Statement::CreateTable(_) => &["create", "table"],
        Statement::CreateIndex(index) if index.unique => &["create", "unique", "index"],
        Statement::CreateIndex(_) => &["create", "index"],
        Statement::CreateTrigger(_) => &["create", "trigger"],
        Statement::CreateFunction(function) if function.or_replace => {
            &["create", "or", "replace", "function"]
        }
        Statement::CreateFunction(_) => &["create", "function"],
        Statement::CreateView(view) if view.materialized => &["create", "materialized", "view"],
        Statement::CreateView(_) => &["create", "view"],
        Statement::CreateType { .. } => &["create", "type"],
        Statement::AlterTable(_) => &["alter", "table"],
        Statement::AlterIndex { .. } => &["alter", "index"],
        _ => return None,
    })
}

fn push_statement(names: &mut Vec<SqlDeclaredIdentifier>, statement: &Statement, line: usize) {
    match statement {
        Statement::CreateTable(table) => {
            push_object(names, &table.name, line);
            for column in &table.columns {
                push_column(names, column, line);
            }
            for constraint in &table.constraints {
                push_constraint(names, constraint, line);
            }
        }
        Statement::CreateIndex(index) => {
            if let Some(name) = &index.name {
                push_object(names, name, line);
            }
        }
        Statement::CreateTrigger(trigger) => push_object(names, &trigger.name, line),
        Statement::CreateFunction(function) => push_object(names, &function.name, line),
        Statement::CreateView(view) => push_object(names, &view.name, line),
        Statement::CreateType { name, .. } => push_object(names, name, line),
        Statement::AlterTable(alter) => {
            for operation in &alter.operations {
                push_alter(names, operation, line);
            }
        }
        Statement::AlterIndex { operation, .. } => {
            let AlterIndexOperation::RenameIndex { index_name } = operation;
            push_object(names, index_name, line);
        }
        _ => {}
    }
}

fn push_alter(
    names: &mut Vec<SqlDeclaredIdentifier>,
    operation: &AlterTableOperation,
    line: usize,
) {
    match operation {
        AlterTableOperation::AddConstraint { constraint, .. } => {
            push_constraint(names, constraint, line);
        }
        AlterTableOperation::AddColumn { column_def, .. } => push_column(names, column_def, line),
        AlterTableOperation::RenameColumn {
            new_column_name, ..
        } => {
            push_ident(names, new_column_name, line);
        }
        AlterTableOperation::RenameTable { table_name } => {
            push_object(names, rename_target(table_name), line);
        }
        AlterTableOperation::RenameConstraint { new_name, .. } => push_ident(names, new_name, line),
        _ => {}
    }
}

fn rename_target(kind: &RenameTableNameKind) -> &ObjectName {
    match kind {
        RenameTableNameKind::As(name) | RenameTableNameKind::To(name) => name,
    }
}

fn push_column(names: &mut Vec<SqlDeclaredIdentifier>, column: &ColumnDef, line: usize) {
    push_ident(names, &column.name, line);
    for option in &column.options {
        if let Some(name) = &option.name {
            push_ident(names, name, line);
        }
    }
}

fn push_constraint(
    names: &mut Vec<SqlDeclaredIdentifier>,
    constraint: &TableConstraint,
    line: usize,
) {
    let Some(name) = constraint_name(constraint) else {
        return;
    };
    push_ident(names, name, line);
}

pub(super) fn constraint_name(constraint: &TableConstraint) -> Option<&sqlparser::ast::Ident> {
    match constraint {
        TableConstraint::Unique(constraint) => constraint.name.as_ref(),
        TableConstraint::PrimaryKey(constraint) => constraint.name.as_ref(),
        TableConstraint::ForeignKey(constraint) => constraint.name.as_ref(),
        TableConstraint::Check(constraint) => constraint.name.as_ref(),
        TableConstraint::Index(constraint) => constraint.name.as_ref(),
        TableConstraint::FulltextOrSpatial(constraint) => constraint.opt_index_name.as_ref(),
        TableConstraint::PrimaryKeyUsingIndex(constraint) => constraint.name.as_ref(),
        TableConstraint::UniqueUsingIndex(constraint) => constraint.name.as_ref(),
        TableConstraint::Exclude(constraint) => constraint.name.as_ref(),
    }
}

fn push_object(names: &mut Vec<SqlDeclaredIdentifier>, name: &ObjectName, line: usize) {
    push_text(names, relation(name), line);
}

fn push_ident(names: &mut Vec<SqlDeclaredIdentifier>, name: &sqlparser::ast::Ident, line: usize) {
    push_text(names, name.value.clone(), line);
}

fn push_text(names: &mut Vec<SqlDeclaredIdentifier>, name: String, line: usize) {
    names.push(SqlDeclaredIdentifier { name, line });
}

#[cfg(test)]
mod tests;
