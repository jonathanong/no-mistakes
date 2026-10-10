//! RETURNING items use the same classes as data-modifying query statements.
use super::super::{
    expressions::{expression, identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::{Insert, Query, SelectItem, SelectItemQualifiedWildcardKind, Spanned};

pub(super) struct ReturningFacts {
    pub items: Vec<PostgresSqlReturningItem>,
    pub complete: bool,
}

pub(super) fn project(items: Option<&[SelectItem]>, locations: &Locations<'_>) -> ReturningFacts {
    let Some(items) = items else {
        return ReturningFacts {
            items: Vec::new(),
            complete: true,
        };
    };
    let mut complete = true;
    let projected = items
        .iter()
        .map(|item| project_item(item, locations, &mut complete))
        .collect();
    ReturningFacts {
        items: projected,
        complete,
    }
}

pub(super) fn diagnostics(
    complete: bool,
    value: &Insert,
    locations: &Locations<'_>,
) -> Vec<PostgresSqlDiagnostic> {
    if complete {
        Vec::new()
    } else {
        vec![PostgresSqlDiagnostic {
            message: "INSERT facts contain unsupported or incompletely represented syntax".into(),
            span: locations.span(value.span()),
        }]
    }
}

/// Column lineage for a RETURNING CTE insert. SELECT facts read the AST body;
/// the placeholder query only carries the source span.
pub(in crate::codebase::postgres::source) fn cte_column_sources(
    columns: &[sqlparser::ast::ObjectName],
    query: Option<&Query>,
    source: &PostgresSqlCteInsertSource,
    locations: &Locations<'_>,
) -> PostgresSqlInsertColumnSources {
    let facts = match source {
        PostgresSqlCteInsertSource::Values { rows, span } => PostgresSqlInsertSource::Values {
            rows: rows.clone(),
            span: span.clone(),
        },
        PostgresSqlCteInsertSource::Select { span, .. } => PostgresSqlInsertSource::Select {
            query: PostgresSqlQuery::default(),
            span: span.clone(),
        },
        PostgresSqlCteInsertSource::DefaultValues => PostgresSqlInsertSource::DefaultValues,
        PostgresSqlCteInsertSource::Unsupported { reason } => {
            PostgresSqlInsertSource::Unsupported {
                reason: reason.clone(),
            }
        }
    };
    super::column_sources::project(columns, query, &facts, &[], locations)
}

fn project_item(
    item: &SelectItem,
    locations: &Locations<'_>,
    complete: &mut bool,
) -> PostgresSqlReturningItem {
    match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            let alias = if let SelectItem::ExprWithAlias { alias, .. } = item {
                Some(identifier(alias))
            } else {
                None
            };
            PostgresSqlReturningItem::Expression {
                expression: Box::new(expression(expr, locations)),
                alias,
            }
        }
        SelectItem::Wildcard(options)
        | SelectItem::QualifiedWildcard(SelectItemQualifiedWildcardKind::ObjectName(_), options)
            if *options == Default::default() =>
        {
            let qualifier = if let SelectItem::QualifiedWildcard(
                SelectItemQualifiedWildcardKind::ObjectName(object),
                _,
            ) = item
            {
                Some(name(object))
            } else {
                None
            };
            PostgresSqlReturningItem::Wildcard {
                qualifier,
                span: locations.span(item.span()),
            }
        }
        _ => {
            *complete = false;
            PostgresSqlReturningItem::Unsupported {
                reason: "RETURNING item".into(),
                span: locations.span(item.span()),
            }
        }
    }
}

#[cfg(test)]
mod tests;
