use oxc_ast::ast::*;

pub(super) fn result_length(
    method: &str,
    length: Option<usize>,
    empty: bool,
    call: &CallExpression<'_>,
) -> Option<usize> {
    if length == Some(0) && matches!(method, "slice" | "splice") {
        return Some(0);
    }
    match method {
        "filter" | "flatMap" | "flat" if empty || length == Some(0) => Some(0),
        "filter" | "flatMap" | "flat" => None,
        "slice" => {
            let len = length?;
            let start = index(call.arguments.first(), len, 0)?;
            let end = index(call.arguments.get(1), len, len)?;
            Some(end.saturating_sub(start))
        }
        "splice" => {
            let len = length?;
            if call.arguments.is_empty() {
                return Some(0);
            }
            let start = index(call.arguments.first(), len, 0)?;
            match call.arguments.get(1) {
                None => Some(len - start),
                Some(argument) => Some(count(argument)?.min(len - start)),
            }
        }
        _ => length,
    }
}

fn index(argument: Option<&Argument<'_>>, len: usize, omitted: usize) -> Option<usize> {
    let Some(argument) = argument else {
        return Some(omitted);
    };
    let value = integer(argument)?;
    if value < 0 {
        Some(if value.unsigned_abs() >= len as u64 {
            0
        } else {
            len - value.unsigned_abs() as usize
        })
    } else {
        Some(if value as u64 >= len as u64 {
            len
        } else {
            value as usize
        })
    }
}

fn count(argument: &Argument<'_>) -> Option<usize> {
    Some((integer(argument)?.max(0) as u64).min(usize::MAX as u64) as usize)
}

fn integer(argument: &Argument<'_>) -> Option<i64> {
    let expression = argument.as_expression()?.without_parentheses();
    let value = match expression {
        Expression::NumericLiteral(number) => number.value,
        Expression::UnaryExpression(unary) => {
            let Expression::NumericLiteral(number) = unary.argument.without_parentheses() else {
                return None;
            };
            match unary.operator {
                UnaryOperator::UnaryNegation => -number.value,
                UnaryOperator::UnaryPlus => number.value,
                _ => return None,
            }
        }
        _ => return None,
    };
    value.is_finite().then_some(value.trunc() as i64)
}
