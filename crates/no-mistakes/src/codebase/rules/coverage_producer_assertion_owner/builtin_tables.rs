//! Name tables for built-in classification; see docs/rules/coverage-producer-assertion-owner.md.

/// Global namespaces whose static methods never reach repository code on their own.
#[rustfmt::skip]
pub(super) const GLOBAL_OBJECTS: &[&str] = &[
    "JSON", "Math", "Number", "String", "Boolean", "BigInt", "Symbol", "Date", "Object", "Array",
    "Promise", "console", "Intl", "RegExp", "Buffer", "URL", "performance",
];
/// Global functions which never reach repository code on their own.
#[rustfmt::skip]
pub(super) const GLOBAL_FUNCTIONS: &[&str] = &[
    "parseInt", "parseFloat", "isNaN", "isFinite", "String", "Number", "Boolean",
    "encodeURIComponent", "decodeURIComponent", "structuredClone",
];
/// Global functions which run their first argument later.
pub(super) const TIMERS: &[&str] = &[
    "setTimeout",
    "setInterval",
    "setImmediate",
    "queueMicrotask",
];
/// String, RegExp, Date, Array, Map, Set, Error, console, and AsyncLocalStorage
/// prototype methods matched by name when the receiver is not statically resolved.
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
    "toFixed", "log", "error", "warn", "info", "debug", "getStore", "enterWith", "disable",
];
/// Methods which invoke a callback at these argument positions.
#[rustfmt::skip]
pub(super) const CALLBACKS: &[(&str, &[usize])] = &[
    ("map", &[0]), ("filter", &[0]), ("forEach", &[0]), ("flatMap", &[0]), ("reduce", &[0]),
    ("reduceRight", &[0]), ("some", &[0]), ("every", &[0]), ("find", &[0]), ("findIndex", &[0]),
    ("findLast", &[0]), ("sort", &[0]), ("toSorted", &[0]), ("then", &[0, 1]), ("catch", &[0]),
    ("finally", &[0]),
];
/// Node modules whose exports are synchronous value computations.
#[rustfmt::skip]
pub(super) const NODE_MODULES: &[&str] = &[
    "path", "node:path", "url", "node:url", "os", "node:os", "querystring", "node:querystring",
    "buffer", "node:buffer", "string_decoder", "node:string_decoder",
];
