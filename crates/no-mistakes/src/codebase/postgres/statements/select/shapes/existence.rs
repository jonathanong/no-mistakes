use super::has_group_by;
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{
    BinaryOperator, Expr, FunctionArguments, Query, Select, SelectItem, SetExpr, Value,
};

pub(super) fn existence(
    op: &BinaryOperator,
    left: &Expr,
    right: &Expr,
    bare: bool,
) -> Option<bool> {
    let (literal_on_right, number) = if is_count_side(left, bare) {
        (true, int_literal(right)?)
    } else if is_count_side(right, bare) {
        (false, int_literal(left)?)
    } else {
        return None;
    };
    pair(op, literal_on_right, number).then_some(matches!(
        (op, literal_on_right, number),
        (BinaryOperator::Eq, _, 0)
            | (BinaryOperator::Lt, true, 1)
            | (BinaryOperator::Gt, false, 1)
            | (BinaryOperator::LtEq, true, 0)
            | (BinaryOperator::GtEq, false, 0)
    ))
}

fn pair(op: &BinaryOperator, literal_on_right: bool, number: u8) -> bool {
    matches!(
        (op, literal_on_right, number),
        (&BinaryOperator::Gt, true, 0)
            | (&BinaryOperator::Lt, false, 0)
            | (&BinaryOperator::GtEq, true, 1)
            | (&BinaryOperator::LtEq, false, 1)
            | (&BinaryOperator::NotEq, _, 0)
            | (&BinaryOperator::Eq, _, 0)
            | (&BinaryOperator::Lt, true, 1)
            | (&BinaryOperator::Gt, false, 1)
            | (&BinaryOperator::LtEq, true, 0)
            | (&BinaryOperator::GtEq, false, 0)
    )
}

fn is_count_side(expr: &Expr, bare: bool) -> bool {
    match unwrap_count(expr) {
        Expr::Subquery(query) => count_query(query),
        Expr::Function(_) if bare => is_count_function(expr),
        _ => false,
    }
}

fn count_query(query: &Query) -> bool {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return false;
    };
    !has_group_by(select) && projection_is_count(select)
}

fn projection_is_count(select: &Select) -> bool {
    let [item] = select.projection.as_slice() else {
        return false;
    };
    match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            is_count_function(expr)
        }
        _ => false,
    }
}

fn is_count_function(expr: &Expr) -> bool {
    let Expr::Function(function) = unwrap_count(expr) else {
        return false;
    };
    let name = function.name.to_string();
    if function.over.is_some()
        || !(name.eq_ignore_ascii_case("count") || name.eq_ignore_ascii_case("pg_catalog.count"))
    {
        return false;
    }
    matches!(function.args, FunctionArguments::List(_))
}

fn unwrap_count(expr: &Expr) -> &Expr {
    match unwrap_expr(expr) {
        Expr::Cast { expr, .. } => unwrap_count(expr),
        other => other,
    }
}

fn int_literal(expr: &Expr) -> Option<u8> {
    let Expr::Value(value) = unwrap_expr(expr) else {
        return None;
    };
    let Value::Number(number, _) = &value.value else {
        return None;
    };
    match number.to_string().as_str() {
        "0" => Some(0),
        "1" => Some(1),
        _ => None,
    }
}
