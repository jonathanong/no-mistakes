use super::super::super::simple_callee_name;
use super::{literal_string, plain_call, RegistryKind};
use oxc_ast::ast::{
    ArrayExpression, Expression, NewExpression, ObjectExpression, ObjectPropertyKind, PropertyKind,
};

pub(super) fn registry(expr: &Expression<'_>) -> Option<(RegistryKind, Vec<String>)> {
    let expr = crate::codebase::ts_source::unwrap_ts_wrappers(expr);
    if let Some(inner) = frozen_argument(expr) {
        return registry(inner);
    }
    match expr {
        Expression::ObjectExpression(object) => {
            object_keys(object).map(|keys| (RegistryKind::Object, keys))
        }
        Expression::ArrayExpression(array) => {
            string_elements(array).map(|keys| (RegistryKind::Array, keys))
        }
        Expression::NewExpression(new) => constructed(new),
        _ => None,
    }
}

fn frozen_argument<'a>(expr: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    let call = plain_call(expr)?;
    if simple_callee_name(&call.callee).as_deref() != Some("Object.freeze")
        || call.arguments.len() != 1
    {
        return None;
    }
    call.arguments[0].as_expression()
}

fn object_keys(object: &ObjectExpression<'_>) -> Option<Vec<String>> {
    let mut keys = Vec::new();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.kind != PropertyKind::Init
            || property.method
            || property.shorthand
            || property.computed
        {
            return None;
        }
        let key = crate::codebase::ts_source::static_property_key_name(&property.key)?;
        literal_string(&property.value)?;
        keys.push(key.into_owned());
    }
    (!keys.is_empty()).then_some(keys)
}

fn string_elements(array: &ArrayExpression<'_>) -> Option<Vec<String>> {
    let mut values = Vec::new();
    for element in &array.elements {
        values.push(literal_string(element.as_expression()?)?);
    }
    (!values.is_empty()).then_some(values)
}

fn constructed(new: &NewExpression<'_>) -> Option<(RegistryKind, Vec<String>)> {
    if new.arguments.len() != 1 {
        return None;
    }
    let name = simple_callee_name(&new.callee)?;
    let Expression::ArrayExpression(array) =
        crate::codebase::ts_source::unwrap_ts_wrappers(new.arguments[0].as_expression()?)
    else {
        return None;
    };
    match name.as_str() {
        "Map" => map_values(array).map(|keys| (RegistryKind::Map, keys)),
        "Set" => string_elements(array).map(|keys| (RegistryKind::Set, keys)),
        _ => None,
    }
}

fn map_values(array: &ArrayExpression<'_>) -> Option<Vec<String>> {
    let mut values = Vec::new();
    for element in &array.elements {
        let Expression::ArrayExpression(pair) =
            crate::codebase::ts_source::unwrap_ts_wrappers(element.as_expression()?)
        else {
            return None;
        };
        if pair.elements.len() != 2 {
            return None;
        }
        literal_string(pair.elements[0].as_expression()?)?;
        values.push(literal_string(pair.elements[1].as_expression()?)?);
    }
    (!values.is_empty()).then_some(values)
}
