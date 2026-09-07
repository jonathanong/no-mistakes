use super::SqlExistsSetOpFact;
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{
    BinaryOperator, Expr, JoinConstraint, JoinOperator, Query, Select, SelectItem, SetExpr, Value,
    ValueWithSpan,
};

pub(super) fn collect_from_select(select: &Select, out: &mut Vec<SqlExistsSetOpFact>) {
    collect_exists(select.selection.as_ref(), out);
    collect_exists(select.having.as_ref(), out);
    for item in &select.projection {
        match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                collect_exists(Some(expr), out);
            }
            _ => {}
        }
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = join_on_expr(&join.join_operator) {
                collect_exists(Some(expr), out);
            }
        }
    }
}

pub(super) fn collect_exists(expr: Option<&Expr>, out: &mut Vec<SqlExistsSetOpFact>) {
    let Some(expr) = expr else {
        return;
    };
    match expr {
        Expr::Exists { subquery, .. } => {
            if set_expr_has_set_op(&subquery.body) {
                out.push(SqlExistsSetOpFact {
                    restricted: set_expr_restricted(&subquery.body),
                    correlated: super::exists_correlation::query_is_correlated(subquery),
                });
            }
            collect_query_exists(subquery, out);
        }
        Expr::BinaryOp { left, right, .. } => {
            collect_exists(Some(left), out);
            collect_exists(Some(right), out);
        }
        Expr::UnaryOp { expr, .. } | Expr::Nested(expr) => collect_exists(Some(expr), out),
        Expr::Subquery(query) => collect_query_exists(query, out),
        _ => {}
    }
}

fn collect_query_exists(query: &Query, out: &mut Vec<SqlExistsSetOpFact>) {
    collect_exists_from_set(&query.body, out);
}

fn collect_exists_from_set(expr: &SetExpr, out: &mut Vec<SqlExistsSetOpFact>) {
    match expr {
        SetExpr::Select(select) => collect_from_select(select, out),
        SetExpr::Query(query) => collect_query_exists(query, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_exists_from_set(left, out);
            collect_exists_from_set(right, out);
        }
        _ => {}
    }
}

fn join_on_expr(operator: &JoinOperator) -> Option<&Expr> {
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

fn set_expr_has_set_op(expr: &SetExpr) -> bool {
    matches!(expr, SetExpr::SetOperation { .. })
        || matches!(expr, SetExpr::Query(query) if set_expr_has_set_op(&query.body))
}

fn set_expr_restricted(expr: &SetExpr) -> bool {
    match expr {
        SetExpr::Select(select) => select.selection.as_ref().is_some_and(expr_is_restricted),
        SetExpr::Query(query) => set_expr_restricted(&query.body),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_restricted(left) && set_expr_restricted(right)
        }
        _ => false,
    }
}

fn expr_is_restricted(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => expr_is_restricted(left) || expr_is_restricted(right),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Or,
            right,
        } => expr_is_restricted(left) && expr_is_restricted(right),
        Expr::BinaryOp { left, right, .. } => column_bound_to_const(left, right),
        _ => false,
    }
}

fn column_bound_to_const(left: &Expr, right: &Expr) -> bool {
    (is_relation_column(left) && is_const_or_placeholder(right))
        || (is_relation_column(right) && is_const_or_placeholder(left))
}

fn is_relation_column(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => !super::value::is_placeholder_ident(&ident.value),
        Expr::CompoundIdentifier(_) => true,
        _ => false,
    }
}

fn is_const_or_placeholder(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(ValueWithSpan { value, .. }) => matches!(
            value,
            Value::Placeholder(_)
                | Value::Number(_, _)
                | Value::SingleQuotedString(_)
                | Value::Boolean(_)
        ),
        Expr::Identifier(ident) => super::value::is_placeholder_ident(&ident.value),
        _ => false,
    }
}
