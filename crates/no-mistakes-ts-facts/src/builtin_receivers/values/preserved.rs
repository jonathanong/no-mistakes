use super::super::names::{PrimitiveKind, ValueKind};
use oxc_ast::ast::*;

pub(super) fn preserved_result(
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
            let result_length = super::array_lengths::result_length(method, length, empty, call);
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
            element: Some(PrimitiveKind::String),
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
