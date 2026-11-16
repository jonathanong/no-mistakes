//! Column bounds from inclusive comparisons and literals.
use super::{days_from_civil, Literal};
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{BinaryOperator, DataType, Expr, UnaryOperator, Value};

pub(super) fn names_column(expr: &Expr, column: &str, qualifiers: &[String]) -> bool {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => ident_key(ident) == column,
        Expr::CompoundIdentifier(parts) => qualified_column(parts, column, qualifiers),
        Expr::Cast {
            expr, data_type, ..
        } if keeps_column(data_type) => names_column(expr, column, qualifiers),
        _ => false,
    }
}

fn qualified_column(parts: &[sqlparser::ast::Ident], column: &str, qualifiers: &[String]) -> bool {
    let Some((name, prefix)) = parts.split_last() else {
        return false;
    };
    !prefix.is_empty()
        && ident_key(name) == column
        && qualifiers
            .iter()
            .any(|qualifier| qualifier.split('.').eq(prefix.iter().map(ident_key)))
}

pub(super) fn absorb(
    column: &str,
    expr: &Expr,
    kind: Literal,
    low: &mut Option<i64>,
    high: &mut Option<i64>,
    qualifiers: &[String],
) {
    let expr = unwrap_expr(expr);
    if let Expr::Between {
        expr,
        negated: false,
        low: lower,
        high: upper,
    } = expr
    {
        if names_column(expr, column, qualifiers) {
            if let Some(value) = literal(lower, kind) {
                raise(low, value);
            }
            if let Some(value) = literal(upper, kind) {
                lower_high(high, value);
            }
        }
        return;
    }
    let Expr::BinaryOp { left, op, right } = expr else {
        return;
    };
    match op {
        BinaryOperator::GtEq if names_column(left, column, qualifiers) => {
            if let Some(value) = literal(right, kind) {
                raise(low, value);
            }
        }
        BinaryOperator::LtEq if names_column(left, column, qualifiers) => {
            if let Some(value) = literal(right, kind) {
                lower_high(high, value);
            }
        }
        BinaryOperator::LtEq if names_column(right, column, qualifiers) => {
            if let Some(value) = literal(left, kind) {
                raise(low, value);
            }
        }
        BinaryOperator::GtEq if names_column(right, column, qualifiers) => {
            if let Some(value) = literal(left, kind) {
                lower_high(high, value);
            }
        }
        _ => {}
    }
}

fn raise(slot: &mut Option<i64>, value: i64) {
    *slot = Some(slot.map_or(value, |current| current.max(value)));
}

fn lower_high(slot: &mut Option<i64>, value: i64) {
    *slot = Some(slot.map_or(value, |current| current.min(value)));
}

fn keeps_column(data_type: &DataType) -> bool {
    super::integer_type(&data_type.to_string()) || matches!(data_type, DataType::Date)
}

fn literal(expr: &Expr, kind: Literal) -> Option<i64> {
    match kind {
        Literal::Integer => integer_literal(expr),
        Literal::Date => {
            date_parts(expr).and_then(|(year, month, day)| days_from_civil(year, month, day))
        }
    }
}

fn integer_literal(expr: &Expr) -> Option<i64> {
    match unwrap_expr(expr) {
        Expr::Value(value) => number(&value.value),
        Expr::UnaryOp {
            op: UnaryOperator::Minus,
            expr,
        } => integer_literal(expr)?.checked_neg(),
        Expr::Cast {
            expr, data_type, ..
        } if super::integer_type(&data_type.to_string()) => {
            integer_literal(expr).or_else(|| quoted_integer(expr))
        }
        _ => None,
    }
}

fn quoted_integer(expr: &Expr) -> Option<i64> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::SingleQuotedString(text) => text.parse().ok(),
            _ => None,
        },
        _ => None,
    }
}

fn number(value: &Value) -> Option<i64> {
    match value {
        Value::Number(text, false) => text.parse().ok(),
        _ => None,
    }
}

fn date_parts(expr: &Expr) -> Option<(i32, u32, u32)> {
    match unwrap_expr(expr) {
        Expr::TypedString(typed) if matches!(typed.data_type, DataType::Date) => {
            date_text(&typed.value.value)
        }
        Expr::Cast {
            expr,
            data_type: DataType::Date,
            ..
        } => match unwrap_expr(expr) {
            Expr::Value(value) => date_text(&value.value),
            _ => None,
        },
        _ => None,
    }
}

fn date_text(value: &Value) -> Option<(i32, u32, u32)> {
    let Value::SingleQuotedString(text) = value else {
        return None;
    };
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year = text.get(..4)?.parse().ok()?;
    let month = text.get(5..7)?.parse().ok()?;
    let day = text.get(8..)?.parse().ok()?;
    Some((year, month, day))
}
