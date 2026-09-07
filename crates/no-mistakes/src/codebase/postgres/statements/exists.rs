use super::SqlExistsSetOpFact;
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr, Query, SetExpr, Value, ValueWithSpan};

pub(super) fn collect_exists(expr: Option<&Expr>, out: &mut Vec<SqlExistsSetOpFact>) {
    let Some(expr) = expr else {
        return;
    };
    match expr {
        Expr::Exists { subquery, .. } => {
            if set_expr_has_set_op(&subquery.body) {
                out.push(SqlExistsSetOpFact {
                    restricted: set_expr_restricted(&subquery.body),
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
        SetExpr::Select(select) => collect_exists(select.selection.as_ref(), out),
        SetExpr::Query(query) => collect_query_exists(query, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_exists_from_set(left, out);
            collect_exists_from_set(right, out);
        }
        _ => {}
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
    match expr {
        Expr::Value(ValueWithSpan {
            value: Value::Placeholder(_),
            ..
        })
        | Expr::Value(ValueWithSpan {
            value: Value::Number(_, _),
            ..
        })
        | Expr::Value(ValueWithSpan {
            value: Value::SingleQuotedString(_),
            ..
        }) => true,
        Expr::Identifier(ident) => super::value::is_placeholder_ident(&ident.value),
        _ => false,
    }
}
