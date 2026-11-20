use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKey,
    PropertyKind,
};
use oxc_span::Span;

pub(super) fn is_named_member(callee: &Expression, object: &str, property: &str) -> bool {
    member_names(callee).is_some_and(|(found_object, found_property)| {
        found_object == object && found_property == property
    })
}

pub(super) fn member_names<'a>(callee: &'a Expression<'a>) -> Option<(&'a str, &'a str)> {
    let Expression::StaticMemberExpression(member) = callee else {
        return None;
    };
    if member.optional {
        return None;
    }
    let Expression::Identifier(object) = &member.object else {
        return None;
    };
    Some((object.name.as_str(), member.property.name.as_str()))
}

pub(super) fn length_expression<'a>(
    object: &'a ObjectExpression<'a>,
) -> Option<&'a Expression<'a>> {
    let mut length = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // A spread can replace `length` at runtime.
            length = None;
            continue;
        };
        match length_key(property) {
            LengthKey::Other => {}
            LengthKey::Dynamic => length = None,
            LengthKey::Named => {
                length = match property.kind {
                    PropertyKind::Init => Some(unwrap_ts_wrappers(&property.value)),
                    PropertyKind::Get | PropertyKind::Set => None,
                };
            }
        }
    }
    length
}

enum LengthKey {
    Named,
    Dynamic,
    Other,
}

fn length_key(property: &ObjectProperty) -> LengthKey {
    match &property.key {
        PropertyKey::StaticIdentifier(identifier) => named_length(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) if literal.value == "length" => LengthKey::Named,
        PropertyKey::StringLiteral(_)
        | PropertyKey::NumericLiteral(_)
        | PropertyKey::PrivateIdentifier(_) => LengthKey::Other,
        _ if property.computed => LengthKey::Dynamic,
        _ => LengthKey::Other,
    }
}

fn named_length(name: &str) -> LengthKey {
    if name == "length" {
        LengthKey::Named
    } else {
        LengthKey::Other
    }
}

pub(super) fn numeric_list(array: &ArrayExpression) -> Vec<(f64, Span)> {
    let mut numbers = Vec::new();
    for element in &array.elements {
        let Some(expr) = element.as_expression() else {
            continue;
        };
        let Expression::NumericLiteral(literal) = unwrap_ts_wrappers(expr) else {
            continue;
        };
        numbers.push((literal.value, literal.span));
    }
    numbers
}

pub(super) fn integer_cardinality(value: f64) -> Option<u64> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > u64::MAX as f64 {
        return None;
    }
    Some(value as u64)
}
