use no_mistakes_postgres_sql::idents::ident_key;
use sqlparser::ast::{Query, Select, SetExpr, Statement};

mod dml;
mod exprs;
mod factors;
mod items;
use dml::{collect_delete, collect_insert, collect_table_expr, collect_update};
use exprs::walk_subqueries;
use factors::{collect_rels, extend_outer};
use items::collect_items;

pub(super) const SYNTHETIC_TARGET: &str = "__nm_frag_target";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FragmentStar {
    pub(super) returning: bool,
    pub(super) qualifier: Option<String>,
    pub(super) relation: Option<String>,
    pub(super) within_function: Option<String>,
    pub(super) line: usize,
}

#[derive(Clone)]
pub(super) struct Rel {
    pub(super) table: String,
    pub(super) alias: Option<String>,
    pub(super) join_aliases: Vec<String>,
    pub(super) is_local_derived: bool,
    pub(super) hidden_by_join_alias: bool,
    pub(super) depth: usize,
}

#[derive(Clone, Copy)]
pub(super) struct Scope<'a> {
    pub(super) ctes: &'a [String],
    pub(super) outer_rels: &'a [Rel],
}

pub(super) fn collect_statements(
    statements: &[Statement],
    in_exists: bool,
    out: &mut Vec<FragmentStar>,
) {
    let scope = Scope {
        ctes: &[],
        outer_rels: &[],
    };
    for statement in statements {
        collect_statement(statement, scope, in_exists, out);
    }
}

fn collect_statement(
    statement: &Statement,
    scope: Scope<'_>,
    in_exists: bool,
    out: &mut Vec<FragmentStar>,
) {
    match statement {
        Statement::Query(query) => collect_query(query, scope, in_exists, out),
        Statement::Insert(insert) => collect_insert(insert, scope, out),
        Statement::Update(update) => collect_update(update, scope, out),
        Statement::Delete(delete) => collect_delete(delete, scope, out),
        Statement::Explain { statement, .. } => collect_statement(statement, scope, false, out),
        _ => {}
    }
}

pub(super) fn collect_query(
    query: &Query,
    scope: Scope<'_>,
    in_exists: bool,
    out: &mut Vec<FragmentStar>,
) {
    let mut ctes = scope.ctes.to_vec();
    if let Some(with) = &query.with {
        if with.recursive {
            for cte in &with.cte_tables {
                ctes.push(ident_key(&cte.alias.name));
            }
        }
        for cte in &with.cte_tables {
            let cte_scope = Scope {
                ctes: &ctes,
                outer_rels: scope.outer_rels,
            };
            collect_query(&cte.query, cte_scope, false, out);
            if !with.recursive {
                ctes.push(ident_key(&cte.alias.name));
            }
        }
    }
    let body_scope = Scope {
        ctes: &ctes,
        outer_rels: scope.outer_rels,
    };
    let simple_rels = collect_set(&query.body, body_scope, in_exists, out);
    if let Some(order_by) = &query.order_by {
        if let sqlparser::ast::OrderByKind::Expressions(exprs) = &order_by.kind {
            let order_outer = extend_outer(&simple_rels, body_scope.outer_rels);
            let order_scope = Scope {
                ctes: body_scope.ctes,
                outer_rels: if simple_rels.is_empty() {
                    body_scope.outer_rels
                } else {
                    &order_outer
                },
            };
            for expr in exprs {
                walk_subqueries(&expr.expr, order_scope, out);
            }
        }
    }
}

fn collect_set(
    expr: &SetExpr,
    scope: Scope<'_>,
    in_exists: bool,
    out: &mut Vec<FragmentStar>,
) -> Vec<Rel> {
    match expr {
        SetExpr::Select(select) => return collect_select(select, scope, in_exists, out),
        SetExpr::Query(query) => collect_query(query, scope, in_exists, out),
        SetExpr::Insert(stmt) | SetExpr::Update(stmt) | SetExpr::Delete(stmt) => {
            collect_statement(stmt, scope, false, out);
        }
        SetExpr::SetOperation {
            op, left, right, ..
        } => {
            let child_exists = in_exists
                && !matches!(
                    op,
                    sqlparser::ast::SetOperator::Intersect | sqlparser::ast::SetOperator::Except
                );
            collect_set(left, scope, child_exists, out);
            collect_set(right, scope, child_exists, out);
        }
        SetExpr::Table(table) if !in_exists => collect_table_expr(table, scope.ctes, out),
        SetExpr::Values(values) => {
            values
                .rows
                .iter()
                .flat_map(|row| row.iter())
                .for_each(|e| walk_subqueries(e, scope, out));
        }
        _ => {}
    }
    Vec::new()
}

fn collect_select(
    select: &Select,
    scope: Scope<'_>,
    in_exists: bool,
    out: &mut Vec<FragmentStar>,
) -> Vec<Rel> {
    let rels = collect_rels(&select.from, scope, out);
    if !in_exists {
        collect_items(&select.projection, &rels, scope, false, out);
    }
    let child_outer = extend_outer(&rels, scope.outer_rels);
    let child_scope = Scope {
        ctes: scope.ctes,
        outer_rels: &child_outer,
    };
    for expr in select
        .selection
        .iter()
        .chain(select.having.iter())
        .chain(select.qualify.iter())
        .chain(select.prewhere.iter())
    {
        walk_subqueries(expr, child_scope, out);
    }
    rels
}
