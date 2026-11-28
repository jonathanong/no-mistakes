mod builtins;
mod conditions;
mod dml;
mod expr_norm;
mod function_clauses;
mod grouping;
mod history;
mod join;
mod nested;
mod normalize;
mod query;
mod scope;
mod select_exprs;
mod subquery;
mod table;
mod windows;

use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use history::LookupHistory;
use nested::{walk_nested_queries, walk_nested_with_context};
use query::walk_query;
use scope::ScopeStack;
use select_exprs::walk_select;
use sqlparser::ast::{CopySource, Expr, SelectItem, SetExpr, Statement};

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

fn walk_set_expr(
    body: &SetExpr,
    order_exprs: &[&Expr],
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> LookupHistory {
    match body {
        SetExpr::Select(select) => walk_select(select, order_exprs, false, scopes, out),
        SetExpr::Query(query) => query::walk_query_ordered(query, order_exprs, scopes, out),
        SetExpr::Values(values) => walk_values(values, usize::MAX, order_exprs, scopes, out),
        SetExpr::SetOperation { left, right, .. } => {
            walk_set_expr(left, &[], scopes, out);
            walk_set_expr(right, &[], scopes, out);
            LookupHistory::new()
        }
        SetExpr::Update(statement)
        | SetExpr::Delete(statement)
        | SetExpr::Insert(statement)
        | SetExpr::Merge(statement) => {
            walk_statement(statement, scopes, out);
            LookupHistory::new()
        }
        _ => LookupHistory::new(),
    }
}

fn walk_values(
    values: &sqlparser::ast::Values,
    row_limit: usize,
    order_by: &[&Expr],
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> LookupHistory {
    scopes.push_frame();
    let mut seen = LookupHistory::new();
    inspect_exprs(
        values
            .rows
            .iter()
            .take(row_limit)
            .flat_map(|row| row.content.iter())
            .chain(order_by.iter().copied()),
        scopes,
        &mut seen,
        out,
    );
    scopes.pop_frame();
    seen
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
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let outer = scopes.snapshot();
    let targets = select_exprs::SelectTargets::default();
    for expr in exprs {
        subquery::inspect_expr(expr, &outer, &targets, seen, out);
        walk_nested_with_context(expr, scopes, out, seen.guard_only(), false, None);
    }
}

#[cfg(test)]
mod tests;
