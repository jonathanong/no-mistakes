use crate::queue::extract_model::EnqueuePriority;
use oxc_ast::ast::{Argument, Expression, ObjectExpression, ObjectPropertyKind, UnaryOperator};
use std::collections::HashMap;

pub(crate) fn const_nonnegative_integer(
    expr: &Expression<'_>,
    numbers: &HashMap<String, f64>,
) -> Option<f64> {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(literal) => nonnegative_int(literal.value),
        Expression::Identifier(identifier) => numbers.get(identifier.name.as_str()).copied(),
        Expression::UnaryExpression(unary) => match unary.operator {
            UnaryOperator::UnaryPlus => const_nonnegative_integer(&unary.argument, numbers),
            UnaryOperator::UnaryNegation => {
                const_nonnegative_integer(&unary.argument, numbers).filter(|value| *value == 0.0)
            }
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn from_call_argument(
    arguments: &[Argument<'_>],
    index: usize,
    numbers: &HashMap<String, f64>,
) -> EnqueuePriority {
    if arguments
        .iter()
        .take(index)
        .any(|argument| matches!(argument, Argument::SpreadElement(_)))
    {
        return EnqueuePriority::Dynamic;
    }
    from_optional_argument(arguments.get(index), numbers)
}

pub(crate) fn from_optional_argument(
    argument: Option<&Argument<'_>>,
    numbers: &HashMap<String, f64>,
) -> EnqueuePriority {
    match argument {
        None => EnqueuePriority::Unprioritized,
        Some(Argument::SpreadElement(_)) => EnqueuePriority::Dynamic,
        Some(argument) => argument
            .as_expression()
            .map(|expr| from_options_expression(expr, numbers))
            .unwrap_or(EnqueuePriority::Dynamic),
    }
}

pub(crate) fn from_job_object(
    object: &ObjectExpression<'_>,
    numbers: &HashMap<String, f64>,
) -> EnqueuePriority {
    own_key(object, "opts", |value| {
        from_options_expression(value, numbers)
    })
}

fn from_options_expression(
    expr: &Expression<'_>,
    numbers: &HashMap<String, f64>,
) -> EnqueuePriority {
    let Expression::ObjectExpression(object) = expr.get_inner_expression() else {
        return EnqueuePriority::Dynamic;
    };
    own_key(object, "priority", |value| classify_value(value, numbers))
}

fn classify_value(expr: &Expression<'_>, numbers: &HashMap<String, f64>) -> EnqueuePriority {
    match const_nonnegative_integer(expr, numbers) {
        Some(0.0) => EnqueuePriority::Unprioritized,
        Some(value) => EnqueuePriority::Prioritized(value),
        None => EnqueuePriority::Dynamic,
    }
}

fn own_key(
    object: &ObjectExpression<'_>,
    key: &str,
    mut classify: impl FnMut(&Expression<'_>) -> EnqueuePriority,
) -> EnqueuePriority {
    let mut found = None;
    let mut open = false;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(_) => {
                found = None;
                open = true;
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some(name) if name == key => {
                        found = Some(classify(property.value.get_inner_expression()));
                        open = false;
                    }
                    Some(_) => {}
                    None => {
                        found = None;
                        open = true;
                    }
                }
            }
        }
    }
    if open {
        EnqueuePriority::Dynamic
    } else {
        found.unwrap_or(EnqueuePriority::Unprioritized)
    }
}

fn nonnegative_int(value: f64) -> Option<f64> {
    (value.is_finite() && value >= 0.0 && value.fract() == 0.0).then_some(value)
}
