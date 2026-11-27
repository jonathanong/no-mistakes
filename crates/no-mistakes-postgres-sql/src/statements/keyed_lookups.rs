mod builtins;
mod dml;
mod expr_norm;
mod join;
mod normalize;
mod scope;
mod select_exprs;
mod subquery;
mod table;
mod windows;

use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use join::register_table_with_joins;
use normalize::LookupKey;
use scope::ScopeStack;
use sqlparser::ast::{
    CopySource, Expr, OrderByKind, Query, Select, SelectItem, SetExpr, Statement, Visit, Visitor,
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
        Statement::Query(query)
        | Statement::Copy {
            source: CopySource::Query(query),
            to: true,
            ..
        } => walk_query(query, scopes, out),
        Statement::Insert(insert) => dml::walk_insert(insert, scopes, out),
        Statement::Update(update) => dml::walk_update(update, scopes, out),
        Statement::Delete(delete) => dml::walk_delete(delete, scopes, out),
        Statement::Merge(merge) => dml::walk_merge(merge, scopes, out),
        Statement::CreateView(view) => walk_query(&view.query, scopes, out),
        Statement::Declare { stmts, .. } => {
            for query in stmts
                .iter()
                .filter_map(|cursor| cursor.for_query.as_deref())
            {
                walk_query(query, scopes, out);
            }
        }
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
    walk_nested_queries(&query.limit_clause, scopes, out);
    walk_nested_queries(&query.fetch, scopes, out);
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
                values
                    .rows
                    .iter()
                    .flat_map(|row| row.content.iter())
                    .chain(order_exprs.iter().copied()),
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
    for table in &select.from {
        register_table_with_joins(table, scopes, &mut seen, out);
    }
    inspect_exprs(
        select_exprs::inspectable(select, order_exprs),
        scopes,
        &mut seen,
        out,
    );
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
        walk_nested_queries(expr, scopes, out);
    }
}

fn walk_nested_queries(
    node: &impl Visit,
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
    let _ = node.visit(&mut visitor);
}

#[cfg(test)]
mod tests;
