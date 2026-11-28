use super::BuiltinType;
use crate::idents::unwrap_expr;
use sqlparser::ast::{Expr, UnaryOperator, Value};

pub(super) fn number_type(number: &str, negate: bool) -> BuiltinType {
    let magnitude = if number.contains('_') {
        number.replace('_', "").parse::<u64>()
    } else {
        number.parse()
    };
    let Ok(magnitude) = magnitude else {
        return BuiltinType::Numeric;
    };
    let signed_headroom = u64::from(negate);
    if magnitude <= i32::MAX as u64 + signed_headroom {
        BuiltinType::Integer
    } else if magnitude <= i64::MAX as u64 + signed_headroom {
        BuiltinType::BigInt
    } else {
        BuiltinType::Numeric
    }
}

pub(super) fn negated_literal_type(expr: &Expr) -> Option<BuiltinType> {
    let mut expr = unwrap_expr(expr);
    let mut negate = true;
    while let Expr::UnaryOp {
        op: UnaryOperator::Minus,
        expr: inner,
    } = expr
    {
        negate = !negate;
        expr = unwrap_expr(inner);
    }
    let Expr::Value(value) = expr else {
        return None;
    };
    if let Value::Number(number, _) = &value.value {
        Some(number_type(number, negate))
    } else {
        None
    }
}
