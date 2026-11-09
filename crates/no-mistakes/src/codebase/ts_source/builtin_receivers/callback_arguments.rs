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
) -> &'static [usize] {
    let empty_array_receiver = matches!(
        names.value_kind(receiver, 0),
        Some(ValueKind::Array { empty: true, .. })
    );
    let empty_array_from = method == "from"
        && call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .is_some_and(|iterable| match iterable.without_parentheses() {
                Expression::ArrayExpression(array) => array.elements.is_empty(),
                _ => matches!(
                    names.value_kind(iterable, 0),
                    Some(ValueKind::Array { empty: true, .. })
                ),
            });
    if empty_array_receiver || empty_array_from {
        return NONE;
    }
    let first =
        positions.contains(&0) && call.arguments.first().is_none_or(argument_may_be_callable);
    let second =
        positions.contains(&1) && call.arguments.get(1).is_none_or(argument_may_be_callable);
    match (first, second) {
        (true, true) => BOTH,
        (true, false) => FIRST,
        (false, true) => SECOND,
        (false, false) => NONE,
    }
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
