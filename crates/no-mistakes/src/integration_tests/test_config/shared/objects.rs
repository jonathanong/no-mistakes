use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, Expression, ObjectExpression, ObjectPropertyKind};
use std::collections::{BTreeMap, BTreeSet};

use super::property_key_name;

mod exports;
pub(in crate::integration_tests) use exports::default_export_object;

pub(in crate::integration_tests) fn expression_config_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match unwrap_ts_wrappers(expression) {
        Expression::CallExpression(call) => call
            .arguments
            .first()
            .and_then(|argument| argument_config_object(argument, bindings, seen)),
        Expression::ObjectExpression(object) => Some(object),
        Expression::Identifier(identifier) => {
            identifier_config_object(identifier.name.as_str(), bindings, seen)
        }
        _ => None,
    }
}

fn argument_config_object<'a>(
    argument: &'a Argument<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => {
            identifier_config_object(identifier.name.as_str(), bindings, seen)
        }
        Argument::ParenthesizedExpression(parenthesized) => {
            expression_config_object(&parenthesized.expression, bindings, seen)
        }
        _ => None,
    }
}

fn identifier_config_object<'a>(
    name: &str,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a ObjectExpression<'a>> {
    if !seen.insert(name.to_string()) {
        return None;
    }
    let object = bindings
        .get(name)
        .and_then(|expression| expression_config_object(expression, bindings, seen));
    seen.remove(name);
    object
}

pub(in crate::integration_tests) fn property_expression_deep<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
) -> Option<&'a Expression<'a>> {
    let mut seen = BTreeSet::new();
    property_expression_deep_inner(object, name, bindings, &mut seen)
}

fn property_expression_deep_inner<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    seen: &mut BTreeSet<String>,
) -> Option<&'a Expression<'a>> {
    let mut found = None;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed || property.method {
                    continue;
                }
                if property_key_name(&property.key).as_deref() == Some(name) {
                    found = Some(&property.value);
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(object) = expression_config_object(&spread.argument, bindings, seen) {
                    if let Some(expression) =
                        property_expression_deep_inner(object, name, bindings, seen)
                    {
                        found = Some(expression);
                    }
                }
            }
        }
    }
    found
}
