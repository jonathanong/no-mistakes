use super::call_shapes::callee_static_member_name;
use crate::ast::js_literal;

use crate::playwright::analysis::text_types::{
    normalize_locator_text, LocatorKind, PlaywrightTextLocator,
};
use crate::playwright::ast;
use options::{object_bool_property, object_has_unsupported_role_filters, BoolProperty};
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind, PropertyKey};

mod options;

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
        "getByAltText" => simple_locator(call, source, LocatorKind::Alt, "getByAltText"),
        "getByTitle" => simple_locator(call, source, LocatorKind::Title, "getByTitle"),
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
        Argument::StringLiteral(literal) => Some(js_literal(literal).into_owned()),
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
        if property.computed
            || property.method
            || property_key_name(&property.key).as_deref() != Some(name)
        {
            continue;
        }
        match &property.value {
            Expression::StringLiteral(literal) => {
                value = normalize_locator_text(js_literal(literal).as_ref());
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
        Expression::StringLiteral(literal) => Some(js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(ast::template_literal_text(template, source))
        }
        _ => None,
    }
}

fn property_key_name<'a>(key: &PropertyKey<'a>) -> Option<std::borrow::Cow<'a, str>> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => {
            Some(std::borrow::Cow::Borrowed(identifier.name.as_str()))
        }
        PropertyKey::StringLiteral(literal) => Some(js_literal(literal)),
        _ => None,
    }
}
