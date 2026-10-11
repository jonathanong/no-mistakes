use super::super::SqlSelectFact;
use sqlparser::ast::{Expr, JoinConstraint, Select, TableFactor, TableWithJoins};

pub(super) fn table_names(from: &[TableWithJoins], ctes: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    for table in from {
        push_table(&table.relation, ctes, &mut names);
        for join in &table.joins {
            push_table(&join.relation, ctes, &mut names);
        }
    }
    names
}

pub(super) fn collect_derived_queries(
    sql: &str,
    from: &[TableWithJoins],
    ctes: &[String],
    in_insert_select: bool,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlSelectFact>,
) {
    for table in from {
        collect_derived_factor(sql, &table.relation, ctes, in_insert_select, positions, out);
        for join in &table.joins {
            collect_derived_factor(sql, &join.relation, ctes, in_insert_select, positions, out);
        }
    }
}

fn collect_derived_factor(
    sql: &str,
    table: &TableFactor,
    ctes: &[String],
    in_insert_select: bool,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlSelectFact>,
) {
    match table {
        TableFactor::Derived { subquery, .. } => {
            super::collect_query_at(sql, subquery, ctes, in_insert_select, false, positions, out);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => collect_derived_queries(
            sql,
            std::slice::from_ref(table_with_joins),
            ctes,
            in_insert_select,
            positions,
            out,
        ),
        _ => {}
    }
}

fn push_table(table: &TableFactor, ctes: &[String], names: &mut Vec<String>) {
    match table {
        TableFactor::Table { name, .. } => {
            if let Some(table) = super::super::predicates::base_table(name, ctes) {
                names.push(table);
            }
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => names.extend(table_names(std::slice::from_ref(table_with_joins), ctes)),
        _ => {}
    }
}

pub(in crate::codebase::postgres::statements) fn predicate_text(select: &Select) -> String {
    let mut parts = Vec::new();
    if let Some(selection) = &select.selection {
        parts.push(selection.to_string());
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = join_expr(&join.join_operator) {
                parts.push(expr.to_string());
            }
        }
    }
    parts.join(" ")
}

pub(super) fn join_expr(operator: &sqlparser::ast::JoinOperator) -> Option<&Expr> {
    match operator {
        sqlparser::ast::JoinOperator::Join(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::Inner(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::Left(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::LeftOuter(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::Right(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::RightOuter(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::FullOuter(JoinConstraint::On(expr)) => Some(expr),
        _ => None,
    }
}
