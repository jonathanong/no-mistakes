//! Name tables for built-in receiver proofs; see docs/rules/coverage-producer-assertion-owner.md.

/// Global namespaces whose static methods never reach repository code on their own.
#[rustfmt::skip]
pub(crate) const GLOBAL_OBJECTS: &[&str] = &[
    "JSON", "Math", "Number", "String", "Boolean", "BigInt", "Symbol", "Date", "Object", "Array",
    "Promise", "console", "Intl", "RegExp", "Buffer", "URL", "performance",
];
/// Namespace methods whose result is a built-in value.
#[rustfmt::skip]
pub(super) const FACTORIES: &[(&str, &str)] = &[
    ("Array", "from"), ("Array", "of"), ("Object", "keys"), ("Object", "values"),
    ("Object", "entries"), ("Object", "fromEntries"), ("Promise", "resolve"),
    ("Promise", "reject"), ("Promise", "all"), ("Promise", "allSettled"), ("Promise", "race"),
    ("Promise", "any"), ("JSON", "stringify"), ("String", "raw"), ("String", "fromCharCode"),
];
/// Global constructors which produce built-in values.
#[rustfmt::skip]
pub(super) const CONSTRUCTORS: &[&str] = &[
    "Map", "Set", "WeakMap", "WeakSet", "Date", "RegExp", "Array", "Promise", "Error", "URL",
    "URLSearchParams", "AsyncLocalStorage", "String", "Number", "Boolean",
];
/// Type names whose values are built-in objects.
#[rustfmt::skip]
pub(super) const TYPES: &[&str] = &[
    "Array", "ReadonlyArray", "Map", "ReadonlyMap", "Set", "ReadonlySet", "WeakMap", "WeakSet",
    "Date", "RegExp", "Promise", "PromiseLike", "Error", "URL", "URLSearchParams",
    "AsyncLocalStorage",
];
/// Methods whose result is again a built-in when the receiver is one.
#[rustfmt::skip]
pub(super) const PRESERVING: &[&str] = &[
    "trim", "trimStart", "trimEnd", "slice", "split", "padStart", "padEnd", "toLowerCase",
    "toUpperCase", "substring", "repeat", "concat", "normalize", "replace", "replaceAll", "join",
    "toString", "map", "filter", "flat", "flatMap", "sort", "toSorted", "reverse", "splice",
    "keys", "values", "entries", "matchAll", "then", "catch", "finally", "set", "add",
];
