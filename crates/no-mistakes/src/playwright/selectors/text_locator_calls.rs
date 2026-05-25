use super::call_shapes::callee_static_member_name;
use crate::playwright::analysis::text_types::{
    normalize_locator_text, LocatorKind, PlaywrightTextLocator,
};
use crate::playwright::ast;
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind, PropertyKey};

pub(super) fn extract_text_locator_call(
    call: &CallExpression<'_>,
    source: &str,
) -> Option<PlaywrightTextLocator> {
    let method = callee_static_member_name(&call.callee)?;
    match method {
        "getByText" => simple_locator(call, source, LocatorKind::Text, "getByText"),
        "getByLabel" => simple_locator(call, source, LocatorKind::Label, "getByLabel"),
        "getByPlaceholder" => {
            simple_locator(call, source, LocatorKind::Placeholder, "getByPlaceholder")
        }
        "getByRole" => role_locator(call, source),
        _ => None,
    }
}

fn simple_locator(
    call: &CallExpression<'_>,
    source: &str,
    kind: LocatorKind,
    method: &str,
) -> Option<PlaywrightTextLocator> {
    let exact = match call.arguments.get(1) {
        Some(argument) if !matches!(argument, Argument::ObjectExpression(_)) => return None,
        Some(argument) => match object_bool_property(argument, "exact") {
            BoolProperty::Unknown => return None,
            BoolProperty::Value(exact) => exact,
            BoolProperty::Missing => false,
        },
        None => false,
    };
    text_arg(call.arguments.first()?, source).map(|text| PlaywrightTextLocator {
        kind,
        role: None,
        locator: format!("{method}({text})"),
        text,
        exact,
        include_hidden: false,
    })
}

fn role_locator(call: &CallExpression<'_>, source: &str) -> Option<PlaywrightTextLocator> {
    let role = text_arg(call.arguments.first()?, source)?;
    let options = call.arguments.get(1)?;
    if object_has_unsupported_role_filters(options) {
        return None;
    }
    let name = object_string_property(options, "name", source)?;
    let exact = match object_bool_property(options, "exact") {
        BoolProperty::Unknown => return None,
        BoolProperty::Value(exact) => exact,
        BoolProperty::Missing => false,
    };
    let include_hidden = match object_bool_property(options, "includeHidden") {
        BoolProperty::Unknown => return None,
        BoolProperty::Value(include_hidden) => include_hidden,
        BoolProperty::Missing => false,
    };
    Some(PlaywrightTextLocator {
        kind: LocatorKind::Role,
        role: Some(role.clone()),
        locator: format!("getByRole({role}, name: {name})"),
        text: name,
        exact,
        include_hidden,
    })
}

fn text_arg(argument: &Argument<'_>, source: &str) -> Option<String> {
    let value = argument_string(argument, source)?;
    normalize_locator_text(&value)
}

fn argument_string(argument: &Argument<'_>, source: &str) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.to_string()),
        Argument::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(ast::template_literal_text(template, source))
        }
        _ => argument
            .as_expression()
            .and_then(|expression| expression_string(expression, source)),
    }
}

fn object_string_property(argument: &Argument<'_>, name: &str, source: &str) -> Option<String> {
    let Argument::ObjectExpression(object) = argument else {
        return None;
    };
    let mut value = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property.method || property_key_name(&property.key) != Some(name) {
            continue;
        }
        match &property.value {
            Expression::StringLiteral(literal) => {
                value = normalize_locator_text(literal.value.as_str());
            }
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                value = normalize_locator_text(&ast::template_literal_text(template, source));
            }
            expression => {
                value = expression_string(expression, source)
                    .and_then(|text| normalize_locator_text(&text));
            }
        }
    }
    value
}

fn expression_string(expression: &Expression<'_>, source: &str) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(ast::template_literal_text(template, source))
        }
        _ => None,
    }
}

fn expression_bool(expression: &Expression<'_>) -> Option<bool> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::BooleanLiteral(literal) => Some(literal.value),
        _ => None,
    }
}

enum BoolProperty {
    Missing,
    Value(bool),
    Unknown,
}

fn object_bool_property(argument: &Argument<'_>, name: &str) -> BoolProperty {
    let Argument::ObjectExpression(object) = argument else {
        return BoolProperty::Missing;
    };
    let mut value = BoolProperty::Missing;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return BoolProperty::Unknown;
        };
        if property.computed || property.method || property_key_name(&property.key) != Some(name) {
            if property.computed || property.method {
                return BoolProperty::Unknown;
            }
            continue;
        }
        if let Some(bool_value) = expression_bool(&property.value) {
            value = BoolProperty::Value(bool_value);
        } else {
            value = BoolProperty::Unknown;
        }
    }
    value
}

fn object_has_unsupported_role_filters(argument: &Argument<'_>) -> bool {
    let Argument::ObjectExpression(object) = argument else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return true;
        };
        if property.computed || property.method {
            return true;
        }
        matches!(
            property_key_name(&property.key),
            Some(
                "checked"
                    | "selected"
                    | "pressed"
                    | "expanded"
                    | "disabled"
                    | "level"
                    | "description"
            )
        )
    })
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}
