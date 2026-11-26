mod expr_norm;
mod join;
mod normalize;
mod scope;
mod subquery;

use crate::codebase::postgres::statement_facts::SqlDuplicateKeyedLookupFact;
use join::{inspect_join, record_lookup, register_table_factor, register_table_with_joins};
use normalize::LookupKey;
use scope::ScopeStack;
use sqlparser::ast::{
    Delete, Expr, FromTable, OrderByKind, Query, Select, SelectItem, SetExpr, Statement, Update,
    Visit, Visitor,
};
use std::collections::BTreeMap;
use std::ops::ControlFlow;
use subquery::collect_expr_lookups;

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
        Statement::Insert(insert) => {
            if let Some(source) = insert.source.as_deref() {
                walk_query(source, scopes, out);
            }
        }
        Statement::Update(update) => walk_update(update, scopes, out),
        Statement::Delete(delete) => walk_delete(delete, scopes, out),
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
        SetExpr::SetOperation { left, right, .. } => {
            walk_set_expr(left, &[], scopes, out);
            walk_set_expr(right, &[], scopes, out);
        }
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
    let exprs = select
        .projection
        .iter()
        .filter_map(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
            _ => None,
        })
        .chain(select.selection.iter())
        .chain(select.having.iter())
        .chain(order_exprs.iter().copied());
    inspect_exprs(exprs, scopes, &mut seen, out);
    scopes.pop_frame();
}

fn walk_update(
    update: &Update,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    scopes.push_frame();
    let mut seen = BTreeMap::new();
    register_table_with_joins(&update.table, scopes, &mut seen, out);
    if let Some(from) = &update.from {
        let tables = match from {
            sqlparser::ast::UpdateTableFromKind::BeforeSet(t)
            | sqlparser::ast::UpdateTableFromKind::AfterSet(t) => t,
        };
        for table in tables {
            register_table_with_joins(table, scopes, &mut seen, out);
        }
    }
    let exprs = update
        .assignments
        .iter()
        .map(|assignment| &assignment.value)
        .chain(update.selection.iter());
    inspect_exprs(exprs, scopes, &mut seen, out);
    scopes.pop_frame();
}

fn walk_delete(
    delete: &Delete,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    scopes.push_frame();
    let mut seen = BTreeMap::new();
    let from_tables = match &delete.from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables,
    };
    for table in from_tables.iter().chain(delete.using.iter().flatten()) {
        register_table_with_joins(table, scopes, &mut seen, out);
    }
    inspect_exprs(delete.selection.iter(), scopes, &mut seen, out);
    scopes.pop_frame();
}

fn inspect_exprs<'a>(
    exprs: impl IntoIterator<Item = &'a Expr>,
    scopes: &mut ScopeStack,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let outer = scopes.snapshot();
    let mut subqueries = Vec::new();
    for expr in exprs {
        collect_expr_lookups(expr, &outer, false, &mut subqueries);
        walk_nested_queries_in_expr(expr, scopes, out);
    }
    for lookup in subqueries {
        let (line, col, cond) = (lookup.line, lookup.column, lookup.conditional);
        record_lookup(lookup.key, lookup.key_display, line, col, cond, seen, out);
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
