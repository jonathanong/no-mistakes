use super::array_escapes::source_array;
use super::names::{Names, ValueKind};
use oxc_ast::ast::*;

const NONE: &[usize] = &[];
const FIRST: &[usize] = &[0];
const SECOND: &[usize] = &[1];
const BOTH: &[usize] = &[0, 1];

pub(super) fn callable_positions(
    names: &Names,
    call: &CallExpression<'_>,
    receiver: &Expression<'_>,
    method: &str,
    positions: &[usize],
) -> Option<&'static [usize]> {
    let uncertain_receiver =
        source_array(receiver).is_some_and(|name| names.unstable_array_lengths.contains(name));
    let uncertain_from = method == "from"
        && call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .and_then(source_array)
            .is_some_and(|name| names.unstable_array_lengths.contains(name));
    // A mutated array is still that array. Follow its callback instead of
    // dropping the call; only a stable empty proof may skip it.
    if !uncertain_receiver && !uncertain_from {
        let holes_visited = matches!(method, "find" | "findIndex" | "findLast" | "findLastIndex");
        if let Some(ValueKind::Array { empty, length, .. }) = names.value_kind(receiver, 0) {
            if length == Some(0) || (empty && !holes_visited) {
                return Some(NONE);
            }
            if empty && length.is_none() && holes_visited {
                return None;
            }
        }
        if method == "from" {
            if let Some(ValueKind::Array { empty, length, .. }) = call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(|iterable| names.value_kind(iterable, 0))
            {
                if length == Some(0) {
                    return Some(NONE);
                }
                if empty && length.is_none() {
                    return None;
                }
            }
        }
    }
    let first =
        positions.contains(&0) && call.arguments.first().is_none_or(argument_may_be_callable);
    let second =
        positions.contains(&1) && call.arguments.get(1).is_none_or(argument_may_be_callable);
    Some(match (first, second) {
        (true, true) => BOTH,
        (true, false) => FIRST,
        (false, true) => SECOND,
        (false, false) => NONE,
    })
}

fn argument_may_be_callable(argument: &Argument<'_>) -> bool {
    let Some(expression) = argument.as_expression() else {
        return true;
    };
    !matches!(
        expression.without_parentheses(),
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
    )
}
