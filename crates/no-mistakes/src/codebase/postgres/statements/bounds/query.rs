use super::functions::{is_aggregate, is_set_returning};
use super::{items, start, Scope};
use crate::codebase::postgres::idents::{ident_key, visit_child_exprs};
use crate::codebase::postgres::statements::limit::is_limited;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::{
    Expr, Function, GroupByExpr, Query, Select, SelectItem, SetExpr, Spanned, Statement,
};

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
                // A recursive reference is whatever the recursion has produced so far: it
                // bounds nothing joined to it.
                inner.insert(name.clone(), opaque(start(cte.query.span())));
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
        SetExpr::Table(table) => super::table::bound(table, start(set.span())),
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

/// A body whose rows nothing proves bounded, and which is never reported either.
fn opaque(at: (usize, usize)) -> SqlBoundQuery {
    SqlBoundQuery {
        capped: false,
        items: vec![items::opaque(at)],
    }
}

/// An aggregate with no `GROUP BY` returns exactly one row, unless a set-returning function in
/// the select list expands it. The aggregate may sit in `HAVING` alone.
fn pure_aggregate(select: &Select) -> bool {
    let ungrouped = matches!(
        &select.group_by,
        GroupByExpr::Expressions(expressions, modifiers)
            if expressions.is_empty() && modifiers.is_empty()
    );
    let projected: Vec<&Expr> = select
        .projection
        .iter()
        .filter_map(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
            _ => None,
        })
        .collect();
    let aggregated = projected
        .iter()
        .any(|expr| contains_call(expr, &is_plain_aggregate))
        || select
            .having
            .as_ref()
            .is_some_and(|having| contains_call(having, &is_plain_aggregate));
    let expanded = projected
        .iter()
        .any(|expr| contains_call(expr, &|function| is_set_returning(&function.name)));
    ungrouped && aggregated && !expanded
}

fn is_plain_aggregate(function: &Function) -> bool {
    function.over.is_none() && is_aggregate(&function.name)
}

/// Whether a call that `test` accepts occurs in `expr` (subqueries are not looked into).
fn contains_call(expr: &Expr, test: &impl Fn(&Function) -> bool) -> bool {
    if let Expr::Function(function) = expr {
        if test(function) {
            return true;
        }
    }
    let mut found = false;
    visit_child_exprs(expr, &mut |child| {
        found = found || contains_call(child, test)
    });
    found
}
