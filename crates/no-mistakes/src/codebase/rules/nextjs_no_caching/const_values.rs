use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{Expression, PropertyKey, TemplateLiteral};

pub(super) type ConstMap = FxHashMap<String, StaticValue>;

#[derive(Clone, Debug)]
pub(super) enum StaticValue {
    String(String),
    Number(f64),
    Bool(bool),
    /// A declared name with no tracked static value. It hides outer constants.
    Unknown,
}

pub(super) fn push_scope(scopes: &mut Vec<ConstMap>) {
    scopes.push(ConstMap::default());
}

pub(super) fn pop_scope(scopes: &mut Vec<ConstMap>) {
    if scopes.len() > 1 {
        scopes.pop();
    }
}

pub(super) fn record_const(scopes: &mut [ConstMap], name: &str, init: &Expression<'_>) {
    let Some(value) = static_value(init, scopes) else {
        record_unknown(scopes, name);
        return;
    };
    insert_new(scopes, name, value);
}

pub(super) fn record_unknown(scopes: &mut [ConstMap], name: &str) {
    insert_new(scopes, name, StaticValue::Unknown);
}

pub(super) fn try_record_resolved(
    scopes: &mut [ConstMap],
    name: &str,
    init: &Expression<'_>,
) -> bool {
    let Some(value) = static_value(init, scopes) else {
        return false;
    };
    insert_new(scopes, name, value);
    true
}

fn insert_new(scopes: &mut [ConstMap], name: &str, value: StaticValue) {
    let Some(scope) = scopes.last_mut() else {
        return;
    };
    scope.entry(name.to_string()).or_insert(value);
}

pub(super) fn static_value(expr: &Expression<'_>, scopes: &[ConstMap]) -> Option<StaticValue> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(StaticValue::String(literal.value.to_string())),
        Expression::BooleanLiteral(literal) => Some(StaticValue::Bool(literal.value)),
        Expression::NumericLiteral(literal) => Some(StaticValue::Number(literal.value)),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(StaticValue::String(template_text(template)))
        }
        Expression::Identifier(identifier) => match lookup(scopes, identifier.name.as_str()) {
            Some(StaticValue::Unknown) | None => None,
            Some(value) => Some(value.clone()),
        },
        _ => None,
    }
}

pub(super) fn lookup<'a>(scopes: &'a [ConstMap], name: &str) -> Option<&'a StaticValue> {
    scopes.iter().rev().find_map(|scope| scope.get(name))
}

pub(super) fn resolved_bool(expr: &Expression<'_>, scopes: &[ConstMap]) -> Option<bool> {
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
