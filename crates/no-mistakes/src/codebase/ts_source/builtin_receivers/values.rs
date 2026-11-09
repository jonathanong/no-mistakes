use super::names::{Names, ValueKind};
use super::tables::{FACTORIES, PRESERVING};
mod array_lengths;
mod type_values;
use oxc_ast::ast::*;
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
                return None;
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
                if let Some(ValueKind::Array { empty, length, .. }) = call
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .and_then(|arg| names.value_kind(arg, depth + 1))
                {
                    result = ValueKind::Array {
                        element: None,
                        empty: length == Some(0) || empty && length.is_none(),
                        length,
                    };
                }
            }
            return Some(result);
        }
    }
    let receiver = names.value_kind(&member.object, depth + 1)?;
    if !PRESERVING.contains(&method) || !names.builtin(&member.object, Some(method), depth + 1) {
        return None;
    }
    preserved_result(receiver, method, call)
}

fn preserved_result(
    receiver: ValueKind,
    method: &str,
    call: &CallExpression<'_>,
) -> Option<ValueKind> {
    match receiver {
        ValueKind::Array { empty, length, .. }
            if matches!(
                method,
                "map"
                    | "filter"
                    | "flatMap"
                    | "flat"
                    | "slice"
                    | "sort"
                    | "toSorted"
                    | "reverse"
                    | "splice"
            ) =>
        {
            let result_length = array_lengths::result_length(method, length, empty, call);
            Some(ValueKind::Array {
                element: None,
                empty: result_length == Some(0) || (empty && method != "toSorted"),
                length: result_length,
            })
        }
        ValueKind::Array { .. } if method == "concat" => Some(ValueKind::Array {
            element: None,
            empty: false,
            length: None,
        }),
        ValueKind::Array { .. } if matches!(method, "join" | "toString") => Some(ValueKind::String),
        ValueKind::Array { .. } if matches!(method, "keys" | "values" | "entries") => {
            Some(ValueKind::Builtin)
        }
        ValueKind::Promise if matches!(method, "then" | "catch" | "finally") => {
            Some(ValueKind::Promise)
        }
        ValueKind::String if method == "split" => Some(ValueKind::Array {
            element: Some(super::names::PrimitiveKind::String),
            empty: false,
            length: None,
        }),
        ValueKind::String
            if matches!(
                method,
                "trim"
                    | "trimStart"
                    | "trimEnd"
                    | "slice"
                    | "padStart"
                    | "padEnd"
                    | "toLowerCase"
                    | "toUpperCase"
                    | "substring"
                    | "repeat"
                    | "concat"
                    | "normalize"
                    | "replace"
                    | "replaceAll"
                    | "toString"
            ) =>
        {
            Some(ValueKind::String)
        }
        ValueKind::String if method == "matchAll" => Some(ValueKind::Builtin),
        ValueKind::Number | ValueKind::Boolean | ValueKind::BigInt if method == "toString" => {
            Some(ValueKind::String)
        }
        ValueKind::Builtin if matches!(method, "set" | "add" | "keys" | "values" | "entries") => {
            Some(ValueKind::Builtin)
        }
        _ => None,
    }
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
