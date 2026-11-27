use super::*;
use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};

type RootValue = (String, u32, ValueExpression);

/// Project the already requested RHS value in target order. This is receiver
/// identity only; it does not add callable value bindings for mutable roots.
pub(super) fn roots<'a>(
    target: &AssignmentTarget<'a>,
    value: ValueExpression,
    expression: &mut impl FnMut(&Expression<'a>) -> ValueExpression,
) -> Vec<RootValue> {
    let mut output = Vec::new();
    walk(target, value, &mut output, expression);
    output
}

fn walk<'a>(
    target: &AssignmentTarget<'a>,
    value: ValueExpression,
    output: &mut Vec<RootValue>,
    expression: &mut impl FnMut(&Expression<'a>) -> ValueExpression,
) {
    match target {
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            let shared = std::sync::Arc::new(value);
            for (index, element) in array.elements.iter().enumerate() {
                if let Some(element) = element {
                    maybe_default(
                        element,
                        project(&shared, &index.to_string()),
                        output,
                        expression,
                    );
                }
            }
            if let Some(rest) = &array.rest {
                walk(
                    &rest.target,
                    array_rest(&shared, array.elements.len()),
                    output,
                    expression,
                );
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            let shared = std::sync::Arc::new(value);
            for property in &object.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                        let selected = project(&shared, property.binding.name.as_str());
                        let value = property.init.as_ref().map_or_else(
                            || selected.clone(),
                            |init| with_default(selected.clone(), expression(init)),
                        );
                        output.push((
                            property.binding.name.to_string(),
                            property.binding.span.start,
                            value,
                        ));
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        let key = if property.computed {
                            property
                                .name
                                .as_expression()
                                .and_then(write_targets::static_key)
                        } else {
                            write_targets::property_key(&property.name)
                        };
                        let value = key
                            .map_or_else(|| possible_fields(&shared), |key| project(&shared, &key));
                        maybe_default(&property.binding, value, output, expression);
                    }
                }
            }
            if let Some(rest) = &object.rest {
                let excluded = object
                    .properties
                    .iter()
                    .filter_map(|property| match property {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                            Some(property.binding.name.to_string())
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                            if property.computed {
                                property
                                    .name
                                    .as_expression()
                                    .and_then(write_targets::static_key)
                            } else {
                                write_targets::property_key(&property.name)
                            }
                        }
                    })
                    .collect::<FxHashSet<_>>();
                walk(
                    &rest.target,
                    object_rest(&shared, &excluded),
                    output,
                    expression,
                );
            }
        }
        _ => {
            if let Some(name) = mutable_receivers::mutable_root_name(target) {
                output.push((name.to_string(), target.span().start, value));
            }
        }
    }
}

fn maybe_default<'a>(
    target: &AssignmentTargetMaybeDefault<'a>,
    value: ValueExpression,
    output: &mut Vec<RootValue>,
    expression: &mut impl FnMut(&Expression<'a>) -> ValueExpression,
) {
    if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) = target {
        let value = with_default(value, expression(&default.init));
        walk(&default.binding, value, output, expression);
    } else if let Some(target) = target.as_assignment_target() {
        walk(target, value, output, expression);
    }
}

fn project(value: &std::sync::Arc<ValueExpression>, key: &str) -> ValueExpression {
    match value.as_ref() {
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => fields
            .get(key)
            .cloned()
            .unwrap_or(ValueExpression::Undefined),
        ValueExpression::Unknown | ValueExpression::Undefined => ValueExpression::Unknown,
        _ => ValueExpression::Member(std::sync::Arc::clone(value), key.to_string()),
    }
}

fn with_default(value: ValueExpression, fallback: ValueExpression) -> ValueExpression {
    match value {
        ValueExpression::Undefined => fallback,
        ValueExpression::Object(_, _)
        | ValueExpression::Array(_)
        | ValueExpression::Callable(_) => value,
        _ => ValueExpression::Default(std::sync::Arc::new(value), std::sync::Arc::new(fallback)),
    }
}

fn possible_fields(value: &ValueExpression) -> ValueExpression {
    let mut values = match value {
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            fields.values().cloned().collect::<Vec<_>>()
        }
        _ => return ValueExpression::Unknown,
    };
    // A wide literal selector creates a balanced possible-value tree, rather
    // than turning source width into recursive reference/drop stack depth.
    while values.len() > 1 {
        let mut next = Vec::with_capacity(values.len().div_ceil(2));
        let mut values_iter = values.into_iter();
        while let Some(left) = values_iter.next() {
            next.push(match values_iter.next() {
                Some(right) => {
                    ValueExpression::Default(std::sync::Arc::new(left), std::sync::Arc::new(right))
                }
                None => left,
            });
        }
        values = next;
    }
    values.pop().unwrap_or(ValueExpression::Unknown)
}

fn array_rest(value: &ValueExpression, start: usize) -> ValueExpression {
    match value {
        ValueExpression::Array(fields) => ValueExpression::Array(std::sync::Arc::new(
            fields
                .iter()
                .filter_map(|(key, value)| {
                    key.parse::<usize>()
                        .ok()
                        .filter(|index| *index >= start)
                        .map(|index| ((index - start).to_string(), value.clone()))
                })
                .collect(),
        )),
        _ => ValueExpression::Unknown,
    }
}
fn object_rest(value: &ValueExpression, excluded: &FxHashSet<String>) -> ValueExpression {
    match value {
        ValueExpression::Object(fields, _) => ValueExpression::Object(
            std::sync::Arc::new(
                fields
                    .iter()
                    .filter(|(key, _)| !excluded.contains(*key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            ),
            false,
        ),
        _ => ValueExpression::Unknown,
    }
}
