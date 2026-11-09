//! Name tables for built-in classification; see docs/rules/coverage-producer-assertion-owner.md.

pub(super) use crate::codebase::ts_source::builtin_receivers::GLOBAL_OBJECTS;
/// Global functions which never reach repository code on their own.
#[rustfmt::skip]
pub(super) const GLOBAL_FUNCTIONS: &[&str] = &[
    "parseInt", "parseFloat", "isNaN", "isFinite", "String", "Number", "Boolean",
    "encodeURIComponent", "decodeURIComponent", "structuredClone",
];
/// Global functions which run their first argument later.
pub(super) const TIMERS: &[&str] = &["setTimeout", "setInterval", "setImmediate"];
/// String, RegExp, Date, Array, Map, Set, Error, console, and AsyncLocalStorage
/// prototype methods accepted only with receiver provenance.
#[rustfmt::skip]
pub(super) const METHODS: &[&str] = &[
    "trim", "trimStart", "trimEnd", "slice", "split", "padStart", "padEnd", "startsWith",
    "endsWith", "includes", "indexOf", "lastIndexOf", "toLowerCase", "toUpperCase", "charAt",
    "charCodeAt", "codePointAt", "substring", "repeat", "concat", "normalize", "localeCompare",
    "at", "join", "push", "pop", "shift", "unshift", "splice", "reverse", "fill", "keys", "values",
    "entries", "flat", "has", "get", "set", "add", "delete", "clear", "test", "exec", "match",
    "matchAll", "toString", "toISOString", "toJSON", "getTime", "getFullYear", "getMonth",
    "getDate", "getDay", "getHours", "getMinutes", "getSeconds", "getMilliseconds",
    "getUTCFullYear", "getUTCMonth", "getUTCDate", "getUTCHours", "getTimezoneOffset", "valueOf",
    "toFixed", "toExponential", "toPrecision", "log", "error", "warn", "info", "debug", "getStore", "enterWith", "disable",
];
/// Node modules whose exports are synchronous value computations.
#[rustfmt::skip]
pub(super) const NODE_MODULES: &[&str] = &[
    "path", "node:path", "url", "node:url", "os", "node:os", "querystring", "node:querystring",
    "buffer", "node:buffer", "string_decoder", "node:string_decoder",
];
