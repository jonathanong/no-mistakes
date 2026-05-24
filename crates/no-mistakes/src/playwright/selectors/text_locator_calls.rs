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
    let value = match argument {
        Argument::StringLiteral(literal) => literal.value.to_string(),
        Argument::TemplateLiteral(template) if template.expressions.is_empty() => {
            ast::template_literal_text(template, source)
        }
        _ => return None,
    };
    normalize_locator_text(&value)
}

fn object_string_property(argument: &Argument<'_>, name: &str, source: &str) -> Option<String> {
    let Argument::ObjectExpression(object) = argument else {
        return None;
    };
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property.method || property_key_name(&property.key) != Some(name) {
            continue;
        }
        match &property.value {
            Expression::StringLiteral(literal) => {
                return normalize_locator_text(literal.value.as_str());
            }
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                let value = ast::template_literal_text(template, source);
                return normalize_locator_text(&value);
            }
            _ => return None,
        }
    }
    None
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
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property.method || property_key_name(&property.key) != Some(name) {
            continue;
        }
        if let Expression::BooleanLiteral(literal) = &property.value {
            return BoolProperty::Value(literal.value);
        }
        return BoolProperty::Unknown;
    }
    BoolProperty::Missing
}

fn object_has_unsupported_role_filters(argument: &Argument<'_>) -> bool {
    let Argument::ObjectExpression(object) = argument else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        matches!(
            property_key_name(&property.key),
            Some("checked" | "selected" | "pressed" | "expanded" | "disabled" | "level")
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
