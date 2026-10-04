use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments};

/// Only known scalar-valued calls preserve a constructor's finite cardinality.
/// Named arguments, windows, aggregates, SRFs and unknown schemas remain opaque.
pub(super) fn arguments(function: &Function) -> Option<Vec<&Expr>> {
    let parts = function
        .name
        .0
        .iter()
        .map(|part| part.as_ident())
        .collect::<Option<Vec<_>>>()?;
    let name = match parts.as_slice() {
        [name] => *name,
        [schema, name] if ident_key(schema) == "pg_catalog" => *name,
        _ => return None,
    };
    let key = ident_key(name);
    // PostgreSQL 15 names may resolve to custom SRFs on older supported servers.
    let later = ["regexp_count", "regexp_instr", "regexp_substr"].contains(&key.as_str());
    if !SCALAR.contains(&key.as_str()) || (later && parts.len() != 2) || function.over.is_some() {
        return None;
    }
    let FunctionArguments::List(arguments) = &function.args else {
        return None;
    };
    arguments
        .args
        .iter()
        .map(|argument| match argument {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => Some(expr),
            _ => None,
        })
        .collect()
}

/// Common PostgreSQL builtins with scalar return cardinality. Unknown names stay opaque;
/// this is an explicit semantic inventory, not a naming convention or catalog fallback.
#[rustfmt::skip]
const SCALAR: &[&str] = &[
    "abs",
    "acos",
    "array_length",
    "array_ndims",
    "array_position",
    "array_to_string",
    "asin",
    "atan",
    "atan2",
    "btrim",
    "cardinality",
    "ceil",
    "ceiling",
    "char_length",
    "character_length",
    "concat",
    "concat_ws",
    "cos",
    "date_part",
    "date_trunc",
    "decode",
    "encode",
    "exp",
    "floor",
    "format",
    "json_array_length",
    "json_build_array",
    "json_build_object",
    "json_extract_path",
    "json_extract_path_text",
    "json_typeof",
    "jsonb_array_length",
    "jsonb_build_array",
    "jsonb_build_object",
    "jsonb_extract_path",
    "jsonb_extract_path_text",
    "jsonb_typeof",
    "length",
    "ln",
    "log",
    "lower",
    "lpad",
    "ltrim",
    "md5",
    "now",
    "octet_length",
    "pg_backend_pid",
    "pg_column_size",
    "pg_typeof",
    "power",
    "quote_ident",
    "quote_literal",
    "quote_nullable",
    "regexp_count",
    "regexp_instr",
    "regexp_replace",
    "regexp_substr",
    "repeat",
    "replace",
    "reverse",
    "round",
    "rpad",
    "rtrim",
    "sign",
    "sin",
    "split_part",
    "sqrt",
    "starts_with",
    "strpos",
    "substr",
    "substring",
    "tan",
    "to_char",
    "to_date",
    "to_json",
    "to_jsonb",
    "to_number",
    "to_timestamp",
    "translate",
    "trunc",
    "upper",
];

#[cfg(test)]
mod tests;
