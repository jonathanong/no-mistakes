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
            continue;
        };
        if is_length_key(property) {
            length = Some(unwrap_ts_wrappers(&property.value));
        }
    }
    length
}

fn is_length_key(property: &ObjectProperty) -> bool {
    if property.kind != PropertyKind::Init {
        return false;
    }
    match &property.key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str() == "length",
        PropertyKey::StringLiteral(literal) => literal.value == "length",
        _ => false,
    }
}

pub(super) fn numeric_list(array: &ArrayExpression) -> Option<Vec<(f64, Span)>> {
    let mut numbers = Vec::new();
    for element in &array.elements {
        let expr = element.as_expression()?;
        let Expression::NumericLiteral(literal) = unwrap_ts_wrappers(expr) else {
            return None;
        };
        numbers.push((literal.value, literal.span));
    }
    Some(numbers)
}

pub(super) fn integer_cardinality(value: f64) -> Option<u64> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > u64::MAX as f64 {
        return None;
    }
    Some(value as u64)
}
