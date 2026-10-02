use super::relation;
use crate::codebase::postgres::types::SqlDeclaredIdentifier;
use sqlparser::ast::{
    AlterIndexOperation, AlterTableOperation, ColumnDef, ObjectName, RenameTableNameKind,
    Statement, TableConstraint,
};

mod locate;
mod procedures;
mod views;

pub(super) use locate::Locations;

pub(super) fn collect(
    locations: &mut Locations,
    statement: &Statement,
) -> Vec<SqlDeclaredIdentifier> {
    let Some((kind, object)) = declared_object(statement) else {
        return Vec::new();
    };
    let Some(line) = locations.take(kind, &relation(object)) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    push_statement(&mut names, statement, line);
    names
}

fn declared_object(statement: &Statement) -> Option<(&'static str, &ObjectName)> {
    Some(match statement {
        Statement::CreateTable(table) => ("table", &table.name),
        Statement::CreateIndex(index) => ("index", index.name.as_ref()?),
        Statement::CreateTrigger(trigger) => ("trigger", &trigger.name),
        Statement::CreateFunction(function) => ("function", &function.name),
        Statement::CreateView(view) => ("view", &view.name),
        Statement::CreateType { name, .. } => ("type", name),
        Statement::AlterTable(table) => ("table", &table.name),
        Statement::AlterIndex { name, .. } => ("index", name),
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
        Statement::CreateView(view) => {
            push_object(names, &view.name, line);
            for column in &view.columns {
                push_ident(names, &column.name, line);
            }
        }
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
