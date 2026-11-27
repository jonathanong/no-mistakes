mod builtins;
mod dml;
mod expr_norm;
mod join;
mod normalize;
mod scope;
mod subquery;

use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use join::{inspect_join, register_table_factor};
use normalize::LookupKey;
use scope::ScopeStack;
use sqlparser::ast::{
    Expr, GroupByExpr, OrderByKind, Query, Select, SelectItem, SetExpr, Statement, Visit, Visitor,
};
use std::collections::BTreeMap;
use std::ops::ControlFlow;

pub(crate) fn collect(statement: &Statement, out: &mut Vec<SqlDuplicateKeyedLookupFact>) {
    let mut scopes = ScopeStack::default();
    walk_statement(statement, &mut scopes, out);
}

fn walk_statement(
    statement: &Statement,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    match statement {
        Statement::Query(query) => walk_query(query, scopes, out),
        Statement::Insert(insert) => dml::walk_insert(insert, scopes, out),
        Statement::Update(update) => dml::walk_update(update, scopes, out),
        Statement::Delete(delete) => dml::walk_delete(delete, scopes, out),
        Statement::Merge(merge) => dml::walk_merge(merge, scopes, out),
        Statement::CreateView(view) => walk_query(&view.query, scopes, out),
        Statement::CreateTable(table) => {
            if let Some(query) = table.query.as_deref() {
                walk_query(query, scopes, out);
            }
        }
        _ => {}
    }
}

fn walk_query(query: &Query, scopes: &mut ScopeStack, out: &mut Vec<SqlDuplicateKeyedLookupFact>) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            walk_query(&cte.query, scopes, out);
        }
    }
    let order_exprs: Vec<&Expr> = match query.order_by.as_ref().map(|order| &order.kind) {
        Some(OrderByKind::Expressions(exprs)) => exprs.iter().map(|e| &e.expr).collect(),
        _ => Vec::new(),
    };
    walk_set_expr(&query.body, &order_exprs, scopes, out);
}

fn walk_set_expr(
    body: &SetExpr,
    order_exprs: &[&Expr],
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    match body {
        SetExpr::Select(select) => walk_select(select, order_exprs, scopes, out),
        SetExpr::Query(query) => walk_query(query, scopes, out),
        SetExpr::Values(values) => {
            scopes.push_frame();
            let mut seen = BTreeMap::new();
            inspect_exprs(
                values.rows.iter().flat_map(|row| row.content.iter()),
                scopes,
                &mut seen,
                out,
            );
            scopes.pop_frame();
        }
        SetExpr::SetOperation { left, right, .. } => {
            walk_set_expr(left, &[], scopes, out);
            walk_set_expr(right, &[], scopes, out);
        }
        SetExpr::Update(statement)
        | SetExpr::Delete(statement)
        | SetExpr::Insert(statement)
        | SetExpr::Merge(statement) => walk_statement(statement, scopes, out),
        _ => {}
    }
}

fn walk_select(
    select: &Select,
    order_exprs: &[&Expr],
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    scopes.push_frame();
    let mut seen = BTreeMap::new();
    for table_with_joins in &select.from {
        register_table_factor(&table_with_joins.relation, scopes, out);
        for join in &table_with_joins.joins {
            inspect_join(join, scopes, &mut seen, out);
        }
    }
    let group_by_exprs: &[Expr] = match &select.group_by {
        GroupByExpr::Expressions(exprs, _) => exprs.as_slice(),
        GroupByExpr::All(_) => &[],
    };
    let exprs = select
        .projection
        .iter()
        .filter_map(select_item_expr)
        .chain(select.selection.iter())
        .chain(group_by_exprs.iter())
        .chain(select.having.iter())
        .chain(order_exprs.iter().copied());
    inspect_exprs(exprs, scopes, &mut seen, out);
    scopes.pop_frame();
}

fn returning_exprs(items: Option<&[SelectItem]>) -> impl Iterator<Item = &Expr> {
    items.into_iter().flatten().filter_map(select_item_expr)
}

fn select_item_expr(item: &SelectItem) -> Option<&Expr> {
    match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
        _ => None,
    }
}

fn inspect_exprs<'a>(
    exprs: impl IntoIterator<Item = &'a Expr>,
    scopes: &mut ScopeStack,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let outer = scopes.snapshot();
    for expr in exprs {
        subquery::inspect_expr(expr, &outer, seen, out);
        walk_nested_queries_in_expr(expr, scopes, out);
    }
}

fn walk_nested_queries_in_expr(
    expr: &Expr,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    struct Nested<'a> {
        depth: usize,
        scopes: &'a mut ScopeStack,
        out: &'a mut Vec<SqlDuplicateKeyedLookupFact>,
    }
    impl Visitor for Nested<'_> {
        type Break = ();
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            if self.depth == 0 {
                walk_query(query, self.scopes, self.out);
            }
            self.depth += 1;
            ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.depth -= 1;
            ControlFlow::Continue(())
        }
    }
    let mut visitor = Nested {
        depth: 0,
        scopes,
        out,
    };
    let _ = expr.visit(&mut visitor);
}

#[cfg(test)]
mod tests;
