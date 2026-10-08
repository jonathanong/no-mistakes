//! Typed statement projections borrow the prepared AST and relationship inputs.
use super::{columns, ddl, expressions::name, indexes, locations::Locations, types::*};
use crate::codebase::postgres::parse::RecursiveViews;
use sqlparser::ast::{Spanned, Statement};

pub(super) fn project_parsed(
    statement: &Statement,
    parser: &sqlparser::parser::Parser<'_>,
    token_range: std::ops::Range<usize>,
    locations: &Locations<'_>,
    recursive_views: &RecursiveViews,
    index_only: bool,
) -> PostgresSqlStatementKind {
    let tokens = if matches!(
        statement,
        Statement::CreateTable(_) | Statement::AlterTable(_)
    ) {
        token_range
            .clone()
            .map(|index| parser.token_at(index).clone())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let tables = crate::codebase::postgres::statements::TableTokenIndex::from_iter(
        token_range.map(|index| parser.token_at(index)),
    );
    project(
        statement,
        locations,
        &tables,
        recursive_views,
        index_only,
        &tokens,
    )
}

pub(super) fn project(
    statement: &Statement,
    locations: &Locations<'_>,
    tables: &crate::codebase::postgres::statements::TableTokenIndex,
    recursive_views: &RecursiveViews,
    index_only: bool,
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
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
        Statement::CreateTable(value) => {
            let definition_ranges = super::constraint_spans::table_ranges(tokens);
            PostgresSqlStatementKind::CreateTable {
                table: name(&value.name),
                columns: value
                    .columns
                    .iter()
                    .map(|column| {
                        columns::column(
                            column,
                            locations,
                            super::constraint_spans::column_span(
                                column,
                                tokens,
                                &definition_ranges,
                                locations,
                            ),
                        )
                    })
                    .collect(),
                constraints: value
                    .constraints
                    .iter()
                    .map(|constraint| {
                        columns::table_constraint(
                            constraint,
                            locations,
                            super::constraint_spans::table_span(
                                constraint,
                                tokens,
                                &definition_ranges,
                                locations,
                            ),
                        )
                    })
                    .collect(),
                temporary: value.temporary,
            }
        }
        Statement::AlterTable(value) => {
            let operation_ranges =
                super::constraint_spans::alter_ranges(tokens, value.name.span().end);
            PostgresSqlStatementKind::AlterTable {
                table: name(&value.name),
                operations: value
                    .operations
                    .iter()
                    .enumerate()
                    .map(|(index, operation)| {
                        let option_spans = match (operation, operation_ranges.get(index)) {
                            (
                                sqlparser::ast::AlterTableOperation::AddColumn {
                                    column_def, ..
                                },
                                Some(range),
                            ) => super::constraint_spans::column_span_in_range(
                                column_def, tokens, *range, locations,
                            ),
                            _ => Vec::new(),
                        };
                        super::alter::alter(
                            operation,
                            locations,
                            super::constraint_spans::alter_span(
                                index,
                                operation,
                                tokens,
                                &operation_ranges,
                                locations,
                            ),
                            option_spans,
                        )
                    })
                    .collect(),
            }
        }
        Statement::CreateIndex(value) => PostgresSqlStatementKind::CreateIndex {
            index: indexes::index(value, locations, index_only),
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
