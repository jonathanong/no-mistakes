use super::{items, start, Scope};
use crate::codebase::postgres::idents::{ident_key, visit_child_exprs};
use crate::codebase::postgres::statements::limit::is_limited;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::{
    Expr, GroupByExpr, ObjectName, Query, Select, SelectItem, SetExpr, Spanned, Statement,
};

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
            sized_by_itself(start(cte.query.span()))
        } else {
            let mut inner = scope.clone();
            if with.recursive {
                // A recursive reference is sized by the recursion, not by this body.
                inner.insert(name.clone(), sized_by_itself(start(cte.query.span())));
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
        _ => sized_by_itself(start(set.span())),
    }
}

fn arm(set: &SetExpr, scope: &Scope) -> SqlBoundItem {
    let (line, column) = start(set.span());
    SqlBoundItem {
        kind: SqlBoundItemKind::Query(set_bound(set, scope)),
        alias: None,
        line,
        column,
        pins: Vec::new(),
    }
}

/// A body whose size nothing in the statement text decides: it adds no unbounded relation.
pub(super) fn sized_by_itself(at: (usize, usize)) -> SqlBoundQuery {
    SqlBoundQuery {
        capped: false,
        items: vec![items::other(at)],
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
        if function.over.is_none() && builtin_aggregate(&function.name) {
            return true;
        }
    }
    let mut found = false;
    visit_child_exprs(expr, &mut |child| {
        found = found || contains_aggregate(child)
    });
    found
}

/// A built-in aggregate: the bare name, or `pg_catalog.<name>`. A function in any other schema
/// that happens to share a name is an ordinary function, called once per row.
fn builtin_aggregate(name: &ObjectName) -> bool {
    let keys: Vec<String> = name
        .0
        .iter()
        .filter_map(|part| part.as_ident().map(ident_key))
        .collect();
    let [.., function] = keys.as_slice() else {
        return false;
    };
    let qualified = match keys.len() {
        1 => true,
        2 => keys[0] == "pg_catalog",
        _ => false,
    };
    qualified && AGGREGATES.contains(&function.as_str())
}
