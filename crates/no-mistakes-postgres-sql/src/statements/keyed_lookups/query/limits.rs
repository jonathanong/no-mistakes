use super::super::conditions::{decimal_integer, literal};
use crate::idents::unwrap_expr;
use sqlparser::ast::{CastKind, DataType, Expr, LimitClause, UnaryOperator, Value};

pub(super) fn discard_exists_limit(limit: Option<&LimitClause>) -> bool {
    match limit {
        None => true,
        Some(LimitClause::LimitOffset {
            limit,
            offset: None,
            ..
        }) => limit.as_ref().is_none_or(|count| {
            matches!(limit_literal(count), Some(Value::Null))
                || literal_limit_value(count).is_some_and(|value| value > 0)
        }),
        _ => false,
    }
}

pub(in crate::statements::keyed_lookups) fn has_zero_limit(limit: Option<&LimitClause>) -> bool {
    let Some(LimitClause::LimitOffset {
        limit: Some(count), ..
    }) = limit
    else {
        return false;
    };
    let count = match unwrap_expr(count) {
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } => expr.as_ref(),
        count => count,
    };
    literal_limit_value(count) == Some(0)
}

fn literal_limit_value(count: &Expr) -> Option<i64> {
    match limit_literal(count)? {
        Value::Number(number, _) => {
            decimal_integer(number).or_else(|| decimal_rounds_to_zero(number).then_some(0))
        }
        Value::SingleQuotedString(number) | Value::EscapedStringLiteral(number) => {
            number.trim_ascii().parse().ok()
        }
        _ => None,
    }
}

fn limit_literal(count: &Expr) -> Option<&Value> {
    let count = match unwrap_expr(count) {
        Expr::Cast {
            kind: CastKind::Cast | CastKind::DoubleColon,
            expr,
            data_type: DataType::BigInt(None),
            format: None,
            ..
        } => expr.as_ref(),
        count => count,
    };
    literal(count)
}

fn decimal_rounds_to_zero(number: &str) -> bool {
    let (mantissa, exponent) = match number.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => {
            let Some(exponent) = decimal_integer(exponent) else {
                return false;
            };
            (mantissa, exponent)
        }
        None => (number, 0),
    };
    let mantissa = mantissa.trim_start_matches(['+', '-']);
    if !mantissa
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'.' | b'_'))
    {
        return false;
    }
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let whole_digits = whole.bytes().filter(u8::is_ascii_digit).count();
    let significant = whole
        .bytes()
        .chain(fraction.bytes())
        .filter(u8::is_ascii_digit)
        .enumerate()
        .find(|(_, digit)| *digit != b'0');
    let Some((leading_zeroes, digit)) = significant else {
        return true;
    };
    let decimal_position = exponent
        .saturating_add(whole_digits as i64)
        .saturating_sub(leading_zeroes as i64);
    decimal_position < 0 || decimal_position == 0 && digit < b'5'
}
