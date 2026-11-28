use no_mistakes_postgres_sql::idents::{ident_key, object_name_key};
use sqlparser::ast::{Query, Select, SetExpr, Statement, TableFactor, TableWithJoins};

mod dml;
mod exprs;
mod items;
use dml::{
    collect_delete, collect_insert, collect_table_expr, collect_update, visit_join_constraint,
};
use exprs::walk_subqueries;
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
}

pub(super) fn collect_statements(statements: &[Statement], out: &mut Vec<FragmentStar>) {
    for statement in statements {
        collect_statement(statement, &[], out);
    }
}

fn collect_statement(statement: &Statement, ctes: &[String], out: &mut Vec<FragmentStar>) {
    match statement {
        Statement::Query(query) => collect_query(query, ctes, false, out),
        Statement::Insert(insert) => collect_insert(insert, ctes, out),
        Statement::Update(update) => collect_update(update, ctes, out),
        Statement::Delete(delete) => collect_delete(delete, ctes, out),
        Statement::Explain { statement, .. } => collect_statement(statement, ctes, out),
        _ => {}
    }
}

pub(super) fn collect_query(
    query: &Query,
    outer_ctes: &[String],
    in_exists: bool,
    out: &mut Vec<FragmentStar>,
) {
    let mut ctes = outer_ctes.to_vec();
    if let Some(with) = &query.with {
        if with.recursive {
            for cte in &with.cte_tables {
                ctes.push(ident_key(&cte.alias.name));
            }
        }
        for cte in &with.cte_tables {
            collect_query(&cte.query, &ctes, false, out);
            if !with.recursive {
                ctes.push(ident_key(&cte.alias.name));
            }
        }
    }
    collect_set(&query.body, &ctes, in_exists, out);
    if let Some(order_by) = &query.order_by {
        if let sqlparser::ast::OrderByKind::Expressions(exprs) = &order_by.kind {
            for expr in exprs {
                walk_subqueries(&expr.expr, &ctes, out);
            }
        }
    }
}

fn collect_set(expr: &SetExpr, ctes: &[String], in_exists: bool, out: &mut Vec<FragmentStar>) {
    match expr {
        SetExpr::Select(select) => collect_select(select, ctes, in_exists, out),
        SetExpr::Query(query) => collect_query(query, ctes, in_exists, out),
        SetExpr::Insert(stmt) | SetExpr::Update(stmt) | SetExpr::Delete(stmt) => {
            collect_statement(stmt, ctes, out);
        }
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(left, ctes, in_exists, out);
            collect_set(right, ctes, in_exists, out);
        }
        SetExpr::Table(table) if !in_exists => collect_table_expr(table, ctes, out),
        SetExpr::Values(values) => {
            values
                .rows
                .iter()
                .flat_map(|row| row.iter())
                .for_each(|e| walk_subqueries(e, ctes, out));
        }
        _ => {}
    }
}

fn collect_select(select: &Select, ctes: &[String], in_exists: bool, out: &mut Vec<FragmentStar>) {
    let rels = collect_rels(&select.from, ctes, out);
    if !in_exists {
        collect_items(&select.projection, &rels, ctes, false, out);
    }
    for expr in select
        .selection
        .iter()
        .chain(select.having.iter())
        .chain(select.qualify.iter())
        .chain(select.prewhere.iter())
    {
        walk_subqueries(expr, ctes, out);
    }
}

pub(super) fn collect_rels(
    from: &[TableWithJoins],
    ctes: &[String],
    out: &mut Vec<FragmentStar>,
) -> Vec<Rel> {
    let mut rels = Vec::new();
    for table in from {
        push_factor(&table.relation, ctes, &mut rels, out);
        for join in &table.joins {
            push_factor(&join.relation, ctes, &mut rels, out);
            visit_join_constraint(&join.join_operator, ctes, out);
        }
    }
    rels
}

pub(super) fn push_factor(
    factor: &TableFactor,
    ctes: &[String],
    rels: &mut Vec<Rel>,
    out: &mut Vec<FragmentStar>,
) {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => {
            let table = object_name_key(name);
            let is_cte = !table.contains('.')
                && ctes.iter().any(|cte| cte.eq_ignore_ascii_case(&table));
            rels.push(Rel {
                table,
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                join_aliases: Vec::new(),
                is_local_derived: is_cte,
            });
        }
        TableFactor::Derived {
            subquery, alias, ..
        } => {
            collect_query(subquery, ctes, false, out);
            rels.push(Rel {
                table: String::new(),
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                join_aliases: Vec::new(),
                is_local_derived: true,
            });
        }
        TableFactor::NestedJoin {
            table_with_joins,
            alias,
        } => {
            let mut inner = collect_rels(std::slice::from_ref(table_with_joins), ctes, out);
            if let Some(alias) = alias {
                let name = ident_key(&alias.name);
                inner
                    .iter_mut()
                    .for_each(|r| r.join_aliases.push(name.clone()));
            }
            rels.extend(inner);
        }
        _ => {}
    }
}
