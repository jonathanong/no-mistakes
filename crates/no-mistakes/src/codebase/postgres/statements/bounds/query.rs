use super::{items, line, Scope};
use crate::codebase::postgres::idents::{ident_key, visit_child_exprs};
use crate::codebase::postgres::statements::limit::is_limited;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::{Expr, GroupByExpr, Query, Select, SelectItem, SetExpr, Spanned, Statement};

const AGGREGATES: &[&str] = &[
    "count",
    "sum",
    "avg",
    "min",
    "max",
    "bool_and",
    "bool_or",
    "every",
    "array_agg",
    "string_agg",
    "json_agg",
    "jsonb_agg",
    "json_object_agg",
    "jsonb_object_agg",
    "xmlagg",
    "bit_and",
    "bit_or",
    "bit_xor",
    "stddev",
    "stddev_pop",
    "stddev_samp",
    "variance",
    "var_pop",
    "var_samp",
    "percentile_cont",
    "percentile_disc",
    "mode",
    "any_value",
    "range_agg",
];

pub(super) fn bound_query(query: &Query, scope: &Scope) -> SqlBoundQuery {
    bound_body(query, &with_scope(query, scope))
}

/// The scope after this query's own `WITH` clause.
pub(super) fn with_scope(query: &Query, outer: &Scope) -> Scope {
    let mut scope = outer.clone();
    let Some(with) = &query.with else {
        return scope;
    };
    for cte in &with.cte_tables {
        let name = ident_key(&cte.alias.name);
        let bound = if modifying_statement(&cte.query).is_some() {
            sized_by_itself(line(cte.query.span()))
        } else {
            let mut inner = scope.clone();
            if with.recursive {
                // A recursive reference is sized by the recursion, not by this body.
                inner.insert(name.clone(), sized_by_itself(line(cte.query.span())));
            }
            bound_query(&cte.query, &inner)
        };
        scope.insert(name, bound);
    }
    scope
}

/// The `INSERT` / `UPDATE` / `DELETE` / `MERGE` inside a data-modifying CTE.
pub(super) fn modifying_statement(query: &Query) -> Option<&Statement> {
    match &*query.body {
        SetExpr::Update(statement)
        | SetExpr::Delete(statement)
        | SetExpr::Insert(statement)
        | SetExpr::Merge(statement) => Some(statement),
        _ => None,
    }
}

/// The body of `query` under `scope`, which already holds its CTEs.
pub(super) fn bound_body(query: &Query, scope: &Scope) -> SqlBoundQuery {
    let mut bound = set_bound(&query.body, scope);
    bound.capped |= is_limited(query);
    bound
}

fn set_bound(set: &SetExpr, scope: &Scope) -> SqlBoundQuery {
    match set {
        SetExpr::Select(select) => SqlBoundQuery {
            capped: pure_aggregate(select),
            items: items::from_select(select, scope),
        },
        SetExpr::Query(query) => bound_query(query, scope),
        // A set operation returns the rows of both arms, so both must be bounded.
        SetExpr::SetOperation { left, right, .. } => SqlBoundQuery {
            capped: false,
            items: vec![arm(left, scope), arm(right, scope)],
        },
        _ => sized_by_itself(line(set.span())),
    }
}

fn arm(set: &SetExpr, scope: &Scope) -> SqlBoundItem {
    SqlBoundItem {
        kind: SqlBoundItemKind::Query(set_bound(set, scope)),
        alias: None,
        line: line(set.span()),
        pins: Vec::new(),
    }
}

/// A body whose size nothing in the statement text decides: it adds no unbounded relation.
pub(super) fn sized_by_itself(line: usize) -> SqlBoundQuery {
    SqlBoundQuery {
        capped: false,
        items: vec![items::other(line)],
    }
}

/// An aggregate with no `GROUP BY` returns exactly one row.
fn pure_aggregate(select: &Select) -> bool {
    let ungrouped = matches!(
        &select.group_by,
        GroupByExpr::Expressions(expressions, modifiers)
            if expressions.is_empty() && modifiers.is_empty()
    );
    ungrouped
        && select.projection.iter().any(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                contains_aggregate(expr)
            }
            _ => false,
        })
}

fn contains_aggregate(expr: &Expr) -> bool {
    if let Expr::Function(function) = expr {
        let name = function
            .name
            .0
            .last()
            .map(|part| part.to_string().to_ascii_lowercase())
            .unwrap_or_default();
        if function.over.is_none() && AGGREGATES.contains(&name.as_str()) {
            return true;
        }
    }
    let mut found = false;
    visit_child_exprs(expr, &mut |child| {
        found = found || contains_aggregate(child)
    });
    found
}
