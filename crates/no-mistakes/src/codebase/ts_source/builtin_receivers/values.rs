use super::names::{Names, ValueKind};
use super::tables::{FACTORIES, PRESERVING};
mod array_lengths;
mod preserved;
mod type_values;
use oxc_ast::ast::*;
use preserved::preserved_result;
pub(super) use type_values::{property_types, value_kind_from_type};

pub(super) const ARRAY_CALLBACKS: &[&str] = &[
    "map",
    "filter",
    "forEach",
    "flatMap",
    "reduce",
    "reduceRight",
    "some",
    "every",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "sort",
    "toSorted",
];

pub(super) fn callback_name(method: &str) -> bool {
    ARRAY_CALLBACKS.contains(&method)
        || matches!(
            method,
            "then"
                | "catch"
                | "finally"
                | "run"
                | "exit"
                | "replace"
                | "replaceAll"
                | "parse"
                | "stringify"
        )
}

pub(super) fn constructor_kind(name: &str) -> ValueKind {
    match name {
        "Array" => ValueKind::Array {
            element: None,
            empty: false,
            length: None,
        },
        "Promise" => ValueKind::Promise,
        "AsyncLocalStorage" => ValueKind::AsyncStorage,
        "String" => ValueKind::String,
        "Number" => ValueKind::Number,
        "Boolean" => ValueKind::Boolean,
        _ => ValueKind::Builtin,
    }
}

pub(super) fn call_result(
    names: &Names,
    call: &CallExpression<'_>,
    depth: u8,
) -> Option<ValueKind> {
    if depth > 8 {
        return None;
    }
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        if let Expression::Identifier(id) = call.callee.without_parentheses() {
            if !names.global(&id.name) {
                return names.function_result(id.name.as_str());
            }
            return match id.name.as_str() {
                "String" => Some(ValueKind::String),
                "Number" => Some(ValueKind::Number),
                "Boolean" => Some(ValueKind::Boolean),
                _ => None,
            };
        }
        return None;
    };
    let method = member.property.name.as_str();
    if matches!(&member.object, Expression::Identifier(id)
        if names.global(&id.name) && id.name == "Date" && method == "UTC")
    {
        return Some(ValueKind::Number);
    }
    if let Expression::Identifier(id) = &member.object {
        if names.global(&id.name) && FACTORIES.contains(&(id.name.as_str(), method)) {
            let mut result = factory_kind(id.name.as_str(), method);
            if id.name == "Array" && method == "from" {
                if let Some(ValueKind::Array { length, .. }) = call
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .and_then(|arg| names.value_kind(arg, depth + 1))
                {
                    result = ValueKind::Array {
                        element: None,
                        empty: length == Some(0),
                        length,
                    };
                }
            }
            return Some(result);
        }
    }
    if method == "toISOString" && super::dates::receiver(names, &member.object) {
        return Some(ValueKind::String);
    }
    let receiver = names.value_kind(&member.object, depth + 1)?;
    if !PRESERVING.contains(&method) || !names.builtin(&member.object, Some(method), depth + 1) {
        return None;
    }
    preserved_result(receiver, method, call)
}

fn factory_kind(owner: &str, method: &str) -> ValueKind {
    match (owner, method) {
        ("Array", _) | ("Object", "keys" | "values" | "entries") => ValueKind::Array {
            element: None,
            empty: false,
            length: None,
        },
        ("Promise", _) => ValueKind::Promise,
        ("String", _) | ("JSON", "stringify") => ValueKind::String,
        _ => ValueKind::Builtin,
    }
}
