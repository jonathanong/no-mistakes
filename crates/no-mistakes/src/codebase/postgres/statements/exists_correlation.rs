//! Approximate correlation: a qualified `table.column` whose qualifier is not
//! among the EXISTS subquery's own FROM/WITH names. Shared UNION-arm locals
//! and nested-subquery scopes are intentionally not modeled.

use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    Expr, JoinConstraint, JoinOperator, Query, Select, SelectItem, SetExpr, TableFactor,
    TableWithJoins,
};
use std::collections::HashSet;

pub(super) fn query_is_correlated(query: &Query) -> bool {
    let mut local = HashSet::new();
    collect_query_locals(query, &mut local);
    query_refs_outside(query, &local)
}

fn collect_query_locals(query: &Query, local: &mut HashSet<String>) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            insert_name(local, &cte.alias.name.value);
        }
    }
    collect_set_locals(&query.body, local);
}

fn collect_set_locals(expr: &SetExpr, local: &mut HashSet<String>) {
    match expr {
        SetExpr::Select(select) => {
            for table in &select.from {
                collect_table_locals(table, local);
            }
        }
        SetExpr::Query(query) => collect_query_locals(query, local),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set_locals(left, local);
            collect_set_locals(right, local);
        }
        _ => {}
    }
}

fn collect_table_locals(table: &TableWithJoins, local: &mut HashSet<String>) {
    collect_factor_locals(&table.relation, local);
    for join in &table.joins {
        collect_factor_locals(&join.relation, local);
    }
}

fn collect_factor_locals(factor: &TableFactor, local: &mut HashSet<String>) {
    match factor {
        TableFactor::Table { name, alias, .. } => {
            if let Some(alias) = alias {
                insert_name(local, &alias.name.value);
            } else {
                insert_name(local, &relation_name(name));
            }
        }
        TableFactor::Derived {
            alias: Some(alias), ..
        } => insert_name(local, &alias.name.value),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => collect_table_locals(table_with_joins, local),
        _ => {}
    }
}

fn insert_name(local: &mut HashSet<String>, name: &str) {
    if !name.is_empty() {
        local.insert(name.to_ascii_lowercase());
    }
}

fn query_refs_outside(query: &Query, local: &HashSet<String>) -> bool {
    query.with.as_ref().is_some_and(|with| {
        with.cte_tables
            .iter()
            .any(|cte| query_refs_outside(&cte.query, local))
    }) || set_refs_outside(&query.body, local)
}

fn set_refs_outside(expr: &SetExpr, local: &HashSet<String>) -> bool {
    match expr {
        SetExpr::Select(select) => select_refs_outside(select, local),
        SetExpr::Query(query) => query_refs_outside(query, local),
        SetExpr::SetOperation { left, right, .. } => {
            set_refs_outside(left, local) || set_refs_outside(right, local)
        }
        _ => false,
    }
}

fn select_refs_outside(select: &Select, local: &HashSet<String>) -> bool {
    select.projection.iter().any(|item| match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            expr_refs_outside(expr, local)
        }
        _ => false,
    }) || select
        .selection
        .as_ref()
        .is_some_and(|expr| expr_refs_outside(expr, local))
        || select
            .having
            .as_ref()
            .is_some_and(|expr| expr_refs_outside(expr, local))
        || select
            .from
            .iter()
            .any(|table| table_refs_outside(table, local))
}

fn table_refs_outside(table: &TableWithJoins, local: &HashSet<String>) -> bool {
    factor_refs_outside(&table.relation, local)
        || table.joins.iter().any(|join| {
            factor_refs_outside(&join.relation, local)
                || join_on(&join.join_operator).is_some_and(|expr| expr_refs_outside(expr, local))
        })
}

fn factor_refs_outside(factor: &TableFactor, local: &HashSet<String>) -> bool {
    match factor {
        TableFactor::Derived { subquery, .. } => query_refs_outside(subquery, local),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => table_refs_outside(table_with_joins, local),
        _ => false,
    }
}

fn join_on(operator: &JoinOperator) -> Option<&Expr> {
    match operator {
        JoinOperator::Join(JoinConstraint::On(expr))
        | JoinOperator::Inner(JoinConstraint::On(expr))
        | JoinOperator::Left(JoinConstraint::On(expr))
        | JoinOperator::LeftOuter(JoinConstraint::On(expr))
        | JoinOperator::Right(JoinConstraint::On(expr))
        | JoinOperator::RightOuter(JoinConstraint::On(expr))
        | JoinOperator::FullOuter(JoinConstraint::On(expr)) => Some(expr),
        _ => None,
    }
}

fn expr_refs_outside(expr: &Expr, local: &HashSet<String>) -> bool {
    match unwrap_expr(expr) {
        Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
            !local.contains(&parts[parts.len() - 2].value.to_ascii_lowercase())
        }
        Expr::Subquery(query)
        | Expr::Exists {
            subquery: query, ..
        }
        | Expr::InSubquery {
            subquery: query, ..
        } => query_refs_outside(query, local),
        other => {
            let mut found = false;
            crate::codebase::postgres::idents::visit_child_exprs(other, &mut |child| {
                if !found {
                    found = expr_refs_outside(child, local);
                }
            });
            found
        }
    }
}

#[cfg(test)]
mod tests;
