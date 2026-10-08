//! INSERT projections consume the same prepared AST as other source facts.
use super::{
    expressions::{expression, identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::{
    AssignmentTarget, Insert, OnConflictAction, OnInsert, SetExpr, Spanned, TableObject,
};
use sqlparser::tokenizer::TokenWithSpan;
mod column_sources;
pub(super) mod parsing;
mod provenance;
pub(super) mod source_projection;
mod spans;
mod targets;
mod values;
use provenance::{provenance, supported_modifiers, syntax_complete};

pub(super) fn project(
    value: &Insert,
    facts: Option<&parsing::ConflictFacts>,
    locations: &Locations<'_>,
    tokens: &[TokenWithSpan],
) -> PostgresSqlInsert {
    project_inner(value, facts, locations, false, tokens)
}

/// The query collector owns CTE sources and RETURNING; borrow only the INSERT core.
pub(super) fn project_cte_core(value: &Insert, locations: &Locations<'_>) -> PostgresSqlInsert {
    project_inner(value, None, locations, true, &[])
}

fn project_inner(
    value: &Insert,
    facts: Option<&parsing::ConflictFacts>,
    locations: &Locations<'_>,
    cte_core: bool,
    tokens: &[TokenWithSpan],
) -> PostgresSqlInsert {
    let table = match &value.table {
        TableObject::TableName(table) => Some(name(table)),
        _ => None,
    };
    let alias = value
        .table_alias
        .as_ref()
        .map(|alias| identifier(&alias.alias));
    let mut complete = table.is_some()
        && supported_modifiers(value, cte_core)
        && !facts.is_some_and(|facts| facts.unsupported_with);
    let source_span = facts.and_then(|facts| facts.source_span);
    let delimiters = spans::locate_delimiters(&spans::delimiters(tokens), locations);
    let source = match value.source.as_ref().filter(|_| !cte_core) {
        None => PostgresSqlInsertSource::DefaultValues,
        Some(query) => match query.body.as_ref() {
            SetExpr::Values(values) => {
                complete &= values::unmodified(query);
                PostgresSqlInsertSource::Values {
                    rows: values
                        .rows
                        .iter()
                        .map(|row| {
                            row.iter()
                                .map(|expr| spans::source_expression(expr, &delimiters, locations))
                                .collect()
                        })
                        .collect(),
                    span: locations.span(source_span.unwrap_or_else(|| query.span())),
                }
            }
            SetExpr::Select(_) | SetExpr::Query(_) | SetExpr::SetOperation { .. } => {
                let facts = super::query::project(query, locations);
                complete &= facts.complete;
                PostgresSqlInsertSource::Select {
                    query: facts,
                    span: locations.span(source_span.unwrap_or_else(|| query.span())),
                }
            }
            _ => {
                complete = false;
                PostgresSqlInsertSource::Unsupported {
                    reason: "Unsupported INSERT source".into(),
                }
            }
        },
    };
    let column_sources = if cte_core {
        PostgresSqlInsertColumnSources::Unsupported {
            reason: PostgresSqlInsertColumnSourcesReason::CteSourceDelegated,
            branch_path: None,
            row_index: None,
            expected_columns: None,
            source_columns: None,
        }
    } else {
        column_sources::project(
            &value.columns,
            value.source.as_deref(),
            &source,
            &delimiters,
            locations,
        )
    };
    let on_conflict = match &value.on {
        Some(OnInsert::OnConflict(conflict)) => {
            let target = targets::conflict(conflict.conflict_target.as_ref(), facts, locations);
            let action = match &conflict.action {
                OnConflictAction::DoNothing => PostgresSqlConflictAction::DoNothing,
                OnConflictAction::DoUpdate(update) => {
                    let assignments = update
                        .assignments
                        .iter()
                        .enumerate()
                        .map(|(index, assignment)| {
                            let (columns, single) = match &assignment.target {
                                AssignmentTarget::ColumnName(column) => (vec![name(column)], true),
                                AssignmentTarget::Tuple(columns) => {
                                    (columns.iter().map(name).collect(), false)
                                }
                            };
                            let provenance =
                                provenance(&assignment.value, table.as_ref(), alias.as_ref());
                            let assignment_facts =
                                facts.and_then(|facts| facts.assignments.get(index));
                            let expression =
                                targets::value(&assignment.value, assignment_facts, locations);
                            let known = single && syntax_complete(&expression.root);
                            complete &= known;
                            let target = assignment_facts
                                .and_then(|facts| facts.target.as_ref())
                                .map(|target| targets::project(target, locations));
                            PostgresSqlInsertAssignment {
                                columns,
                                target,
                                expression,
                                provenance,
                                span: locations.span(
                                    assignment_facts
                                        .map(|facts| facts.span)
                                        .unwrap_or_else(|| assignment.span()),
                                ),
                                complete: known,
                            }
                        })
                        .collect();
                    PostgresSqlConflictAction::DoUpdate {
                        assignments,
                        predicate: update.selection.as_ref().map(|expr| {
                            Box::new(
                                match facts.and_then(|facts| facts.action_predicate.as_ref()) {
                                    Some((span, delimiters)) => {
                                        spans::expression(expr, *span, delimiters, locations)
                                    }
                                    None => expression(expr, locations),
                                },
                            )
                        }),
                    }
                }
            };
            Some(PostgresSqlConflict {
                target,
                predicate: facts
                    .and_then(|facts| facts.predicate.as_ref())
                    .map(|expr| targets::located(expr, locations)),
                action,
                span: locations.span(
                    facts
                        .and_then(|facts| facts.span)
                        .unwrap_or_else(|| conflict.span()),
                ),
            })
        }
        None => None,
        _ => {
            complete = false;
            None
        }
    };
    let diagnostics = if complete {
        Vec::new()
    } else {
        vec![PostgresSqlDiagnostic {
            message: "INSERT facts contain unsupported or incompletely represented syntax".into(),
            span: locations.span(value.span()),
        }]
    };
    PostgresSqlInsert {
        table,
        alias,
        columns: value.columns.iter().map(name).collect(),
        columns_omitted: value.columns.is_empty(),
        column_sources,
        source,
        on_conflict,
        span: locations.span(value.span()),
        complete,
        diagnostics,
    }
}

pub(super) fn values_unmodified(query: &sqlparser::ast::Query) -> bool {
    values::unmodified(query)
}
