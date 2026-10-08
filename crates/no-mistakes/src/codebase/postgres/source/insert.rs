//! INSERT projections consume the same prepared AST as other source facts.
use super::{
    expressions::{expression, identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::{
    AssignmentTarget, Insert, OnConflictAction, OnInsert, SetExpr, Spanned, TableObject,
};
pub(super) mod parsing;
mod provenance;
mod spans;
mod targets;
mod values;
use provenance::{provenance, syntax_complete};

pub(super) fn project(
    value: &Insert,
    facts: Option<&parsing::ConflictFacts>,
    locations: &Locations<'_>,
) -> PostgresSqlInsert {
    project_inner(value, facts, locations, false)
}

/// The query collector owns CTE sources and RETURNING; borrow only the INSERT core.
pub(super) fn project_cte_core(value: &Insert, locations: &Locations<'_>) -> PostgresSqlInsert {
    project_inner(value, None, locations, true)
}

fn project_inner(
    value: &Insert,
    facts: Option<&parsing::ConflictFacts>,
    locations: &Locations<'_>,
    cte_core: bool,
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
    let source = match value.source.as_ref().filter(|_| !cte_core) {
        None => PostgresSqlInsertSource::DefaultValues,
        Some(query) => match query.body.as_ref() {
            SetExpr::Values(values) => {
                complete &= values::unmodified(query);
                PostgresSqlInsertSource::Values {
                    rows: values
                        .rows
                        .iter()
                        .map(|row| row.iter().map(|expr| expression(expr, locations)).collect())
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
                        predicate: update
                            .selection
                            .as_ref()
                            .map(|expr| Box::new(expression(expr, locations))),
                    }
                }
            };
            Some(PostgresSqlConflict {
                target,
                predicate: facts
                    .and_then(|facts| facts.predicate.as_ref())
                    .map(|expr| expression(expr, locations)),
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
        source,
        on_conflict,
        span: locations.span(value.span()),
        complete,
        diagnostics,
    }
}

fn supported_modifiers(value: &Insert, cte_core: bool) -> bool {
    value.or.is_none()
        && !value.ignore
        && !value.overwrite
        && !value.has_table_keyword
        && value.assignments.is_empty()
        && value.partitioned.is_none()
        && value.after_columns.is_empty()
        && (value.returning.is_none() || cte_core)
        && value.output.is_none()
        && !value.replace_into
        && value.priority.is_none()
        && value.insert_alias.is_none()
        && value.settings.is_none()
        && value.format_clause.is_none()
        && value.multi_table_insert_type.is_none()
        && value.multi_table_into_clauses.is_empty()
        && value.multi_table_when_clauses.is_empty()
        && value.multi_table_else_clause.is_none()
}

pub(super) fn values_unmodified(query: &sqlparser::ast::Query) -> bool {
    values::unmodified(query)
}
