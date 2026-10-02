mod from;
mod nested;
mod shapes;
mod stars;
mod uses;
pub(super) use nested::walk_node;

pub(super) fn join_expr(operator: &sqlparser::ast::JoinOperator) -> Option<&sqlparser::ast::Expr> {
    from::join_expr(operator)
}

pub(super) fn collect_predicate_shapes(expr: &sqlparser::ast::Expr, out: &mut Vec<SqlSelectFact>) {
    let shapes = shapes::collect_predicate(expr);
    if shapes.not_in_subqueries.is_empty() && shapes.count_existence_checks.is_empty() {
        return;
    }
    out.push(SqlSelectFact {
        line: 1,
        tables: Vec::new(),
        predicate_sql: String::new(),
        exists_set_operations: Vec::new(),
        relations: Vec::new(),
        in_insert_select: false,
        not_in_subqueries: shapes.not_in_subqueries,
        not_in_columns: shapes.not_in_columns,
        count_existence_checks: shapes.count_existence_checks,
        star_projections: Vec::new(),
        column_uses: Vec::new(),
    });
}

/// Record `EXISTS (... UNION ...)` facts for expressions that are not part of a SELECT.
pub(super) fn collect_exists_facts(
    sql: &str,
    expr: &sqlparser::ast::Expr,
    out: &mut Vec<SqlSelectFact>,
) {
    let mut exists_set_operations = Vec::new();
    super::exists::collect_exists(sql, Some(expr), &mut exists_set_operations);
    if exists_set_operations.is_empty() {
        return;
    }
    out.push(SqlSelectFact {
        line: 1,
        tables: Vec::new(),
        predicate_sql: String::new(),
        exists_set_operations,
        relations: Vec::new(),
        in_insert_select: false,
        not_in_subqueries: Vec::new(),
        not_in_columns: Vec::new(),
        count_existence_checks: Vec::new(),
        star_projections: Vec::new(),
        column_uses: Vec::new(),
    });
}

pub(super) fn walk_expr(
    sql: &str,
    expr: &sqlparser::ast::Expr,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    nested::walk_expr(sql, expr, ctes, in_insert_select, out);
}

use super::{SqlSelectFact, SqlStarProjectionFact};
use sqlparser::ast::{Query, Select, SetExpr, Statement};

pub(super) fn mutation_column_uses(
    tables: &[sqlparser::ast::TableWithJoins],
    selection: Option<&sqlparser::ast::Expr>,
    ctes: &[String],
) -> Vec<super::SqlColumnUseFact> {
    uses::collect_mutation(tables, selection, ctes)
}

pub(super) fn returning_stars(sql: &str, statement: &Statement) -> Vec<SqlStarProjectionFact> {
    stars::returning(sql, statement)
}

pub(super) fn collect(sql: &str, statement: &Statement, out: &mut Vec<SqlSelectFact>) {
    match statement {
        Statement::Query(query) => collect_query(sql, query, &[], false, false, out),
        Statement::Insert(insert) => {
            if let Some(source) = insert.source.as_deref() {
                collect_query(sql, source, &[], true, false, out);
            }
        }
        Statement::CreateView(view) => collect_query(sql, &view.query, &[], false, false, out),
        Statement::CreateTable(table) => {
            if let Some(query) = table.query.as_deref() {
                collect_query(sql, query, &[], false, false, out);
            }
        }
        Statement::Copy {
            source: sqlparser::ast::CopySource::Query(query),
            ..
        } => collect_query(sql, query, &[], false, false, out),
        Statement::Explain { statement, .. } => collect(sql, statement, out),
        _ => {}
    }
}

pub(super) fn collect_query(
    sql: &str,
    query: &Query,
    outer_ctes: &[String],
    in_insert_select: bool,
    in_exists: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    let mut ctes = outer_ctes.to_vec();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            let name = crate::codebase::postgres::idents::ident_key(&cte.alias.name);
            if with.recursive {
                ctes.push(name.clone());
            }
            collect_query(sql, &cte.query, &ctes, in_insert_select, false, out);
            if !with.recursive {
                ctes.push(name);
            }
        }
    }
    nested::collect_query_expressions(sql, query, &ctes, in_insert_select, out);
    let order = query_order(query);
    collect_set(
        sql,
        &query.body,
        &ctes,
        in_insert_select,
        in_exists,
        order,
        out,
    );
}

fn query_order(query: &Query) -> &[sqlparser::ast::OrderByExpr] {
    match query.order_by.as_ref().map(|order| &order.kind) {
        Some(sqlparser::ast::OrderByKind::Expressions(exprs)) => exprs,
        _ => &[],
    }
}

fn collect_set(
    sql: &str,
    expr: &SetExpr,
    ctes: &[String],
    in_insert_select: bool,
    in_exists: bool,
    order: &[sqlparser::ast::OrderByExpr],
    out: &mut Vec<SqlSelectFact>,
) {
    match expr {
        SetExpr::Select(select) => {
            push_select(sql, select, ctes, in_insert_select, in_exists, order, out);
        }
        SetExpr::Query(query) => collect_query(sql, query, ctes, in_insert_select, in_exists, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(sql, left, ctes, in_insert_select, in_exists, &[], out);
            collect_set(sql, right, ctes, in_insert_select, in_exists, &[], out);
        }
        SetExpr::Values(values) => {
            for expr in values.rows.iter().flat_map(|row| row.iter()) {
                walk_expr(sql, expr, ctes, in_insert_select, out);
            }
        }
        SetExpr::Insert(Statement::Insert(insert)) => {
            if let Some(source) = insert.source.as_deref() {
                collect_query(sql, source, ctes, true, false, out);
            }
        }
        _ => {}
    }
}

fn push_select(
    sql: &str,
    select: &Select,
    ctes: &[String],
    in_insert_select: bool,
    in_exists: bool,
    order: &[sqlparser::ast::OrderByExpr],
    out: &mut Vec<SqlSelectFact>,
) {
    let tables = from::table_names(&select.from, ctes);
    let mut exists_set_operations = Vec::new();
    super::exists::collect_from_select(sql, select, &mut exists_set_operations);
    from::collect_derived_queries(sql, &select.from, ctes, in_insert_select, out);
    nested::collect(sql, select, ctes, in_insert_select, out);
    let relations = super::predicates::select_relations(sql, select, ctes);
    let shapes = shapes::collect(select);
    let line = super::lines::line_containing(
        sql,
        &[tables.first().map(String::as_str).unwrap_or("select")],
    );
    let star_projections = if in_exists {
        Vec::new()
    } else {
        stars::collect(select, ctes, line)
    };
    let column_uses = uses::collect(select, order, ctes, line);
    if tables.is_empty()
        && exists_set_operations.is_empty()
        && relations.is_empty()
        && shapes.not_in_subqueries.is_empty()
        && shapes.count_existence_checks.is_empty()
        && star_projections.is_empty()
        && column_uses.is_empty()
    {
        return;
    }
    out.push(SqlSelectFact {
        line,
        tables,
        predicate_sql: from::predicate_text(select),
        exists_set_operations,
        relations,
        in_insert_select,
        not_in_subqueries: shapes.not_in_subqueries,
        not_in_columns: shapes.not_in_columns,
        count_existence_checks: shapes.count_existence_checks,
        star_projections,
        column_uses,
    });
}

#[cfg(test)]
mod tests;
