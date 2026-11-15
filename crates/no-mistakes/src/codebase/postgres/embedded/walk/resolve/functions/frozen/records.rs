use super::super::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::helpers::program::{FrozenEntry, FrozenField};
use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKind};

#[derive(Clone)]
pub(super) enum Values {
    Strings(Vec<(String, RecoveredSql)>),
    Objects(Vec<FrozenEntry>),
}

pub(super) fn record_values(object: &ObjectExpression<'_>, source: &str) -> Option<Values> {
    if object.properties.is_empty() || object.properties.len() > 16 {
        return None;
    }
    let mut strings = Vec::new();
    let mut objects = Vec::new();
    for property in &object.properties {
        let (key, value) = field(property)?;
        if let Some(inner) = nested_record(value) {
            objects.push((key, nested(inner, source)?));
        } else {
            strings.push((key, sql_of(value, source)?));
        }
    }
    if !strings.is_empty() && !objects.is_empty() {
        return None;
    }
    if !strings.is_empty() {
        return unique_keys(&strings).then_some(Values::Strings(strings));
    }
    object_entries(objects)
}

/// The outer `Object.freeze(...)` object, after one layer of type wrappers.
pub(super) fn frozen_object<'a>(expr: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    match unwrap_ts_wrappers(freeze_argument(call)?) {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn nested_record<'a>(value: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    let value = unwrap_ts_wrappers(value);
    if let Expression::ObjectExpression(object) = value {
        return Some(object);
    }
    frozen_object(value)
}

fn freeze_argument<'a>(call: &'a oxc_ast::ast::CallExpression<'a>) -> Option<&'a Expression<'a>> {
    if call.optional || call.arguments.len() != 1 {
        return None;
    }
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    if member.optional || object.name != "Object" || member.property.name != "freeze" {
        return None;
    }
    call.arguments[0].as_expression()
}

fn field<'a>(property: &'a ObjectPropertyKind<'a>) -> Option<(String, &'a Expression<'a>)> {
    let ObjectPropertyKind::ObjectProperty(property) = property else {
        return None;
    };
    if property.computed || property.method || property.kind != PropertyKind::Init {
        return None;
    }
    let key = static_property_key_name(&property.key)?;
    if key.is_empty() || key == "__proto__" {
        return None;
    }
    Some((key.into_owned(), &property.value))
}

fn nested(object: &ObjectExpression<'_>, source: &str) -> Option<Vec<(String, RecoveredSql)>> {
    if object.properties.is_empty() {
        return None;
    }
    let mut fields = Vec::new();
    for property in &object.properties {
        let (key, value) = field(property)?;
        fields.push((key, sql_of(value, source)?));
    }
    unique_keys(&fields).then_some(fields)
}

fn object_entries(objects: Vec<(String, Vec<(String, RecoveredSql)>)>) -> Option<Values> {
    let mut seen = FxHashSet::default();
    if !objects.iter().all(|(key, _)| seen.insert(key.clone())) {
        return None;
    }
    let names: Vec<String> = objects
        .first()?
        .1
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    let mut entries = Vec::new();
    for (key, fields) in objects {
        if fields.len() != names.len() {
            return None;
        }
        let mut ordered = Vec::new();
        for name in &names {
            let sql = fields.iter().find(|(field, _)| field == name)?.1.clone();
            ordered.push(FrozenField {
                name: name.clone(),
                sql,
            });
        }
        entries.push(FrozenEntry {
            key,
            fields: ordered,
        });
    }
    Some(Values::Objects(entries))
}

fn unique_keys<T>(pairs: &[(String, T)]) -> bool {
    let mut seen = FxHashSet::default();
    pairs.iter().all(|(key, _)| seen.insert(key.clone()))
}

fn sql_of(expr: &Expression<'_>, source: &str) -> Option<RecoveredSql> {
    let expr = unwrap_ts_wrappers(expr);
    let text = match expr {
        Expression::StringLiteral(literal) => crate::ast::js_literal(literal).into_owned(),
        Expression::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            crate::ast::template_quasi_text(&template.quasis[0]).into_owned()
        }
        _ => return None,
    };
    if text.is_empty() || text.contains(['\0', '\n', '\r']) || sentinel::contains(&text) {
        return None;
    }
    Some(RecoveredSql::literal(expr, source, text))
}
