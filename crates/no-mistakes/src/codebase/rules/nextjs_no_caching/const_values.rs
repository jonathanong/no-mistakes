use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, PropertyKey, TemplateLiteral};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(super) enum StaticValue {
    String(String),
    Number(f64),
    Bool(bool),
}

pub(super) fn push_scope(scopes: &mut Vec<HashMap<String, StaticValue>>) {
    scopes.push(HashMap::new());
}

pub(super) fn pop_scope(scopes: &mut Vec<HashMap<String, StaticValue>>) {
    if scopes.len() > 1 {
        scopes.pop();
    }
}

pub(super) fn record_const(
    scopes: &mut [HashMap<String, StaticValue>],
    name: &str,
    init: &Expression<'_>,
) {
    let Some(value) = static_value(init, scopes) else {
        return;
    };
    let Some(scope) = scopes.last_mut() else {
        return;
    };
    scope.entry(name.to_string()).or_insert(value);
}

pub(super) fn static_value(
    expr: &Expression<'_>,
    scopes: &[HashMap<String, StaticValue>],
) -> Option<StaticValue> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(StaticValue::String(literal.value.to_string())),
        Expression::BooleanLiteral(literal) => Some(StaticValue::Bool(literal.value)),
        Expression::NumericLiteral(literal) => Some(StaticValue::Number(literal.value)),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(StaticValue::String(template_text(template)))
        }
        Expression::Identifier(identifier) => lookup(scopes, identifier.name.as_str()).cloned(),
        _ => None,
    }
}

pub(super) fn lookup<'a>(
    scopes: &'a [HashMap<String, StaticValue>],
    name: &str,
) -> Option<&'a StaticValue> {
    scopes.iter().rev().find_map(|scope| scope.get(name))
}

pub(super) fn resolved_bool(
    expr: &Expression<'_>,
    scopes: &[HashMap<String, StaticValue>],
) -> Option<bool> {
    match static_value(expr, scopes)? {
        StaticValue::Bool(value) => Some(value),
        _ => None,
    }
}

pub(super) fn property_name(key: &PropertyKey<'_>) -> Option<String> {
    if let Some(name) = crate::codebase::ts_source::static_property_key_name(key) {
        return Some(name.to_string());
    }
    let PropertyKey::TemplateLiteral(template) = key else {
        return None;
    };
    if !template.expressions.is_empty() {
        return None;
    }
    Some(template_text(template))
}

fn template_text(template: &TemplateLiteral<'_>) -> String {
    template
        .quasis
        .iter()
        .map(|quasi| {
            quasi
                .value
                .cooked
                .as_ref()
                .unwrap_or(&quasi.value.raw)
                .as_str()
        })
        .collect()
}
