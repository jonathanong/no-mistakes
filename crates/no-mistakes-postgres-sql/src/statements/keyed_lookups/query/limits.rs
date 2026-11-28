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

pub(super) fn literal_limit_value(count: &Expr) -> Option<i64> {
    match limit_literal(count)? {
        Value::Number(number, _) => decimal_integer(number).or_else(|| decimal_to_i64(number)),
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

fn decimal_to_i64(number: &str) -> Option<i64> {
    let (mantissa, exponent) = match number.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, decimal_integer(exponent)?),
        None => (number, 0),
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((whole, fraction)) if !fraction.contains('.') => (whole, fraction),
        Some(_) => return None,
        None => (mantissa, ""),
    };
    if !whole
        .bytes()
        .chain(fraction.bytes())
        .all(|byte| byte.is_ascii_digit() || byte == b'_')
    {
        return None;
    }
    let whole_digits = whole.bytes().filter(u8::is_ascii_digit).count();
    let digits: Vec<u8> = whole
        .bytes()
        .chain(fraction.bytes())
        .filter(u8::is_ascii_digit)
        .collect();
    if digits.is_empty() {
        return None;
    }
    let Some(first_nonzero) = digits.iter().position(|&digit| digit != b'0') else {
        return Some(0);
    };
    let significant = &digits[first_nonzero..];
    let decimal_position = exponent
        .saturating_add(whole_digits as i64)
        .saturating_sub(first_nonzero as i64);
    if decimal_position < 0 {
        return Some(0);
    }
    if decimal_position > 19 {
        return None;
    }
    let integer_digits = decimal_position as usize;
    let mut value: u128 = 0;
    for index in 0..integer_digits {
        let digit = significant.get(index).copied().unwrap_or(b'0') - b'0';
        value = value * 10 + u128::from(digit);
    }
    if significant.get(integer_digits).copied().unwrap_or(b'0') >= b'5' {
        value += 1;
    }
    i64::try_from(value).ok()
}
