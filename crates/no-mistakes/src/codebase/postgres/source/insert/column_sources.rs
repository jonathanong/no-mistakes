mod branches;
#[cfg(test)]
mod tests;
use super::{super::locations::Locations, super::types::*};
use branches::{collect, validate_arity, Branch};
use sqlparser::ast::{Query, SetExpr};
use std::collections::BTreeSet;

pub(super) fn project(
    columns: &[sqlparser::ast::ObjectName],
    source: Option<&Query>,
    source_facts: &PostgresSqlInsertSource,
    delimiters: &[PostgresSqlSpan],
    locations: &Locations<'_>,
) -> PostgresSqlInsertColumnSources {
    if columns.is_empty() {
        return unsupported(PostgresSqlInsertColumnSourcesReason::ColumnsOmitted);
    }
    if source.is_none() {
        return unsupported(PostgresSqlInsertColumnSourcesReason::DefaultValues);
    }
    if duplicate_columns(columns) {
        return unsupported(PostgresSqlInsertColumnSourcesReason::DuplicateTargetColumn);
    }

    let mut branches = Vec::new();
    if let Err(reason) = collect(&source.unwrap().body, Vec::new(), 0, &mut branches) {
        return reason;
    }
    if let Some(reason) = validate_arity(columns.len(), &branches) {
        return reason;
    }

    let mut mapped = Vec::with_capacity(columns.len());
    let mut complete = true;
    for (column_index, column) in columns.iter().enumerate() {
        let mut sources = Vec::new();
        for branch in &branches {
            match branch {
                Branch::Values { path, rows } => {
                    for (row_index, row) in rows.iter().enumerate() {
                        let value = if path.is_empty()
                            && matches!(source.unwrap().body.as_ref(), SetExpr::Values(_))
                        {
                            if let PostgresSqlInsertSource::Values { rows, .. } = source_facts {
                                rows.get(row_index)
                                    .and_then(|row| row.get(column_index))
                                    .cloned()
                            } else {
                                None
                            }
                        } else {
                            row.get(column_index).map(|expr| {
                                super::spans::source_expression(expr, delimiters, locations)
                            })
                        };
                        let Some(value) = value else {
                            return unsupported(
                                PostgresSqlInsertColumnSourcesReason::UnsupportedSource,
                            );
                        };
                        complete &= value.children_complete;
                        sources.push(PostgresSqlInsertSourceExpression::Values {
                            branch_path: path.clone(),
                            row_index,
                            expression: value,
                        });
                    }
                }
                Branch::Select { path, expressions } => {
                    let value = super::spans::source_expression(
                        expressions[column_index],
                        delimiters,
                        locations,
                    );
                    complete &= value.children_complete;
                    sources.push(PostgresSqlInsertSourceExpression::Select {
                        branch_path: path.clone(),
                        expression: value,
                    });
                }
            }
        }
        mapped.push(PostgresSqlInsertColumnSource {
            column_index,
            column: super::super::expressions::name(column),
            sources,
        });
    }

    PostgresSqlInsertColumnSources::Mapped {
        columns: mapped,
        complete,
    }
}

fn duplicate_columns(columns: &[sqlparser::ast::ObjectName]) -> bool {
    let mut identities = BTreeSet::new();
    for column in columns {
        let identity = super::super::expressions::name(column)
            .parts
            .into_iter()
            .map(|part| part.identity)
            .collect::<Vec<_>>();
        if !identities.insert(identity) {
            return true;
        }
    }
    false
}

fn unsupported(reason: PostgresSqlInsertColumnSourcesReason) -> PostgresSqlInsertColumnSources {
    unsupported_details(reason, None, None, None, None)
}

fn unsupported_details(
    reason: PostgresSqlInsertColumnSourcesReason,
    branch_path: Option<Vec<usize>>,
    row_index: Option<usize>,
    expected_columns: Option<usize>,
    source_columns: Option<usize>,
) -> PostgresSqlInsertColumnSources {
    PostgresSqlInsertColumnSources::Unsupported {
        reason,
        branch_path,
        row_index,
        expected_columns,
        source_columns,
    }
}
