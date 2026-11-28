use crate::codebase::dependencies::extract::{ImportedBinding, ValueExpression};
use crate::fx::FxHashMap;

#[derive(Clone, Copy)]
pub(super) struct BuiltinCall {
    pub(super) callbacks: &'static [usize],
    pub(super) optional_non_callable: bool,
}

impl BuiltinCall {
    const LEAF: Self = Self {
        callbacks: &[],
        optional_non_callable: false,
    };
}

#[rustfmt::skip]
pub(super) const GLOBAL_STATIC_OBJECTS: &[&str] = &[
    "Array", "BigInt", "Boolean", "Buffer", "Date", "Error", "Intl", "JSON", "Map", "Math",
    "Number", "Object", "Promise", "RegExp", "Set", "String", "Symbol", "URL", "console",
    "performance",
];

#[rustfmt::skip]
const GLOBAL_CONSTRUCTORS: &[&str] = &[
    "Array", "Boolean", "Date", "Error", "Map", "Number", "Promise", "RegExp", "Set", "String",
    "URL", "URLSearchParams", "WeakMap", "WeakSet",
];

#[rustfmt::skip]
const GLOBAL_FUNCTIONS: &[&str] = &[
    "Array", "BigInt", "Boolean", "Date", "Error", "Number", "Object", "RegExp", "String",
    "Symbol", "atob", "btoa", "decodeURI", "decodeURIComponent", "encodeURI",
    "encodeURIComponent", "isFinite", "isNaN", "parseFloat", "parseInt", "structuredClone",
];

#[rustfmt::skip]
const PROTO_LEAF_METHODS: &[&str] = &[
    "add", "append", "at", "charAt", "charCodeAt", "clear", "codePointAt", "compare", "concat",
    "copyWithin", "delete", "difference", "endsWith", "entries", "equals", "exec", "fill", "flat",
    "get", "getAll", "getDate", "getDay", "getFullYear", "getHours", "getMilliseconds",
    "getMinutes", "getMonth", "getSeconds", "getTime", "getTimezoneOffset", "getUTCDate",
    "getUTCDay", "getUTCFullYear", "getUTCHours", "getUTCMilliseconds", "getUTCMinutes",
    "getUTCSeconds", "has", "hasOwnProperty", "includes", "indexOf", "intersection",
    "isDisjointFrom", "isPrototypeOf", "isSubsetOf", "isSupersetOf", "isWellFormed", "join",
    "keys", "lastIndexOf", "localeCompare", "match", "matchAll", "normalize", "padEnd",
    "padStart", "pop", "propertyIsEnumerable", "push", "repeat", "reverse", "search", "set",
    "setDate", "setFullYear", "setHours", "setMilliseconds", "setMinutes", "setMonth",
    "setSeconds", "setTime", "setUTCDate", "setUTCFullYear", "setUTCHours", "setUTCMilliseconds",
    "setUTCMinutes", "setUTCSeconds", "shift", "slice", "splice", "split", "startsWith",
    "subarray", "substring", "symmetricDifference", "test", "toDateString", "toExponential",
    "toFixed", "toISOString", "toJSON", "toLocaleDateString", "toLocaleLowerCase",
    "toLocaleString", "toLocaleTimeString", "toLocaleUpperCase", "toLowerCase", "toPrecision",
    "toReversed", "toSpliced", "toString", "toTimeString", "toUTCString", "toUpperCase",
    "toWellFormed", "trim", "trimEnd", "trimLeft", "trimRight", "trimStart", "union", "unshift",
    "valueOf", "values", "with",
];

pub(super) fn global_static_call(object: &str, method: &str) -> Option<BuiltinCall> {
    if !GLOBAL_STATIC_OBJECTS.contains(&object) {
        return None;
    }
    Some(match (object, method) {
        ("Array", "from" | "fromAsync") | ("Object", "groupBy") | ("Map", "groupBy") => {
            BuiltinCall {
                callbacks: &[1],
                optional_non_callable: false,
            }
        }
        ("JSON", "parse") => BuiltinCall {
            callbacks: &[1],
            optional_non_callable: true,
        },
        ("JSON", "stringify") => BuiltinCall {
            callbacks: &[1],
            optional_non_callable: true,
        },
        ("Promise", "try") => BuiltinCall {
            callbacks: &[0],
            optional_non_callable: false,
        },
        _ => BuiltinCall::LEAF,
    })
}

pub(super) fn global_target_callbacks(name: &str) -> Option<BuiltinCall> {
    let trimmed = name
        .split_once('.')
        .filter(|(root, _)| matches!(*root, "globalThis" | "window" | "self" | "global"))
        .map_or(name, |(_, rest)| rest);
    let (object, method) = trimmed.split_once('.')?;
    global_static_call(object, method).filter(|call| !call.callbacks.is_empty())
}

pub(super) fn proto_method_call(method: &str) -> Option<BuiltinCall> {
    match method {
        "map" | "flatMap" | "filter" | "reduce" | "reduceRight" | "forEach" | "some" | "every"
        | "find" | "findIndex" | "findLast" | "findLastIndex" | "sort" | "toSorted" | "catch"
        | "finally" => Some(BuiltinCall {
            callbacks: &[0],
            optional_non_callable: false,
        }),
        "then" => Some(BuiltinCall {
            callbacks: &[0, 1],
            optional_non_callable: false,
        }),
        "replace" | "replaceAll" => Some(BuiltinCall {
            callbacks: &[1],
            optional_non_callable: true,
        }),
        _ if PROTO_LEAF_METHODS.contains(&method) => Some(BuiltinCall::LEAF),
        _ => None,
    }
}

pub(super) fn is_builtin_constructor(name: &str) -> bool {
    GLOBAL_CONSTRUCTORS.contains(&name)
}

pub(super) fn is_builtin_global_fn(
    expr: &ValueExpression,
    imports: &FxHashMap<&str, &ImportedBinding>,
) -> bool {
    match expr {
        ValueExpression::Named {
            name, scope: None, ..
        } => !imports.contains_key(name.as_str()) && GLOBAL_FUNCTIONS.contains(&name.as_str()),
        ValueExpression::Member(receiver, method) => match receiver.as_ref() {
            ValueExpression::Named {
                name, scope: None, ..
            } if !imports.contains_key(name.as_str())
                && GLOBAL_STATIC_OBJECTS.contains(&name.as_str()) =>
            {
                global_static_call(name, method).is_some_and(|call| call.callbacks.is_empty())
            }
            _ => false,
        },
        _ => false,
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "builtins_tests.rs"]
mod tests;
