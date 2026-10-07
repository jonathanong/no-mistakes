//! Typed statement projections borrow the prepared AST and relationship inputs.
use super::{columns, ddl, expressions::name, indexes, locations::Locations, types::*};
use crate::codebase::postgres::parse::RecursiveViews;
use sqlparser::ast::Statement;

pub(super) fn project(
    statement: &Statement,
    locations: &Locations<'_>,
    tables: &crate::codebase::postgres::statements::TableTokenIndex,
    recursive_views: &RecursiveViews,
) -> PostgresSqlStatementKind {
    if let Some(drop) = super::drop_facts::drop(statement) {
        return PostgresSqlStatementKind::Drop { drop };
    }
    match statement {
        Statement::Insert(value) => PostgresSqlStatementKind::Insert {
            insert: Box::new(super::insert::project(value, None, locations)),
        },
        Statement::Query(query) => PostgresSqlStatementKind::Select {
            query: super::query::project(query, locations),
        },
        Statement::CreateTable(value) => PostgresSqlStatementKind::CreateTable {
            table: name(&value.name),
            columns: value
                .columns
                .iter()
                .map(|column| columns::column(column, locations))
                .collect(),
            constraints: value
                .constraints
                .iter()
                .map(|constraint| columns::table_constraint(constraint, locations))
                .collect(),
            temporary: value.temporary,
        },
        Statement::AlterTable(value) => PostgresSqlStatementKind::AlterTable {
            table: name(&value.name),
            operations: value
                .operations
                .iter()
                .map(|operation| super::alter::alter(operation, locations))
                .collect(),
        },
        Statement::CreateIndex(value) => PostgresSqlStatementKind::CreateIndex {
            index: indexes::index(value, locations),
        },
        Statement::CreateView(value) => PostgresSqlStatementKind::CreateView {
            view: ddl::view(value, locations, tables, recursive_views.contains(value)),
        },
        Statement::CreateTrigger(value) => PostgresSqlStatementKind::CreateTrigger {
            trigger: ddl::trigger(value, locations),
        },
        Statement::CreateFunction(value) => PostgresSqlStatementKind::CreateFunction {
            function: ddl::function(value, locations),
        },
        _ => PostgresSqlStatementKind::Other,
    }
}
