use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments};

/// Only explicitly qualified builtin calls preserve a constructor's finite cardinality.
/// Unqualified names can resolve to custom overloads, even at a builtin's usual arity.
pub(super) fn arguments(function: &Function) -> Option<Vec<&Expr>> {
    let parts = function
        .name
        .0
        .iter()
        .map(|part| part.as_ident())
        .collect::<Option<Vec<_>>>()?;
    let [schema, name] = parts.as_slice() else {
        return None;
    };
    if ident_key(schema) != "pg_catalog" || function.over.is_some() {
        return None;
    }
    let key = ident_key(name);
    let (_, min, max) = SIGNATURES.iter().find(|(name, _, _)| *name == key)?;
    let FunctionArguments::List(arguments) = &function.args else {
        return None;
    };
    if !(*min..=*max).contains(&arguments.args.len()) {
        return None;
    }
    arguments
        .args
        .iter()
        .map(|argument| match argument {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => Some(expr),
            _ => None,
        })
        .collect()
}

/// Supported scalar builtin arities, including explicit zero-argument and variadic forms.
/// Qualification prevents absent overloads from resolving to application functions.
#[rustfmt::skip]
const SIGNATURES: &[(&str, usize, usize)] = &[
    ("abs", 1, 1),
    ("acos", 1, 1),
    ("array_length", 2, 2),
    ("array_ndims", 1, 1),
    ("array_position", 2, 3),
    ("array_to_string", 2, 3),
    ("asin", 1, 1),
    ("atan", 1, 1),
    ("atan2", 2, 2),
    ("btrim", 1, 2),
    ("cardinality", 1, 1),
    ("ceil", 1, 1),
    ("ceiling", 1, 1),
    ("char_length", 1, 1),
    ("character_length", 1, 1),
    ("concat", 1, usize::MAX),
    ("concat_ws", 2, usize::MAX),
    ("cos", 1, 1),
    ("date_part", 2, 2),
    ("date_trunc", 2, 3),
    ("decode", 2, 2),
    ("encode", 2, 2),
    ("exp", 1, 1),
    ("floor", 1, 1),
    ("format", 1, usize::MAX),
    ("json_array_length", 1, 1),
    ("json_build_array", 0, usize::MAX),
    ("json_build_object", 0, usize::MAX),
    ("json_extract_path", 2, usize::MAX),
    ("json_extract_path_text", 2, usize::MAX),
    ("json_typeof", 1, 1),
    ("jsonb_array_length", 1, 1),
    ("jsonb_build_array", 0, usize::MAX),
    ("jsonb_build_object", 0, usize::MAX),
    ("jsonb_extract_path", 2, usize::MAX),
    ("jsonb_extract_path_text", 2, usize::MAX),
    ("jsonb_typeof", 1, 1),
    ("length", 1, 2),
    ("ln", 1, 1),
    ("log", 1, 2),
    ("lower", 1, 1),
    ("lpad", 2, 3),
    ("ltrim", 1, 2),
    ("md5", 1, 1),
    ("now", 0, 0),
    ("octet_length", 1, 1),
    ("pg_backend_pid", 0, 0),
    ("pg_column_size", 1, 1),
    ("pg_typeof", 1, 1),
    ("power", 2, 2),
    ("quote_ident", 1, 1),
    ("quote_literal", 1, 1),
    ("quote_nullable", 1, 1),
    ("regexp_count", 2, 4),
    ("regexp_instr", 2, 7),
    ("regexp_replace", 3, 6),
    ("regexp_substr", 2, 6),
    ("repeat", 2, 2),
    ("replace", 3, 3),
    ("reverse", 1, 1),
    ("round", 1, 2),
    ("rpad", 2, 3),
    ("rtrim", 1, 2),
    ("sign", 1, 1),
    ("sin", 1, 1),
    ("split_part", 3, 3),
    ("sqrt", 1, 1),
    ("starts_with", 2, 2),
    ("strpos", 2, 2),
    ("substr", 2, 3),
    ("substring", 2, 3),
    ("tan", 1, 1),
    ("to_char", 2, 2),
    ("to_date", 2, 2),
    ("to_json", 1, 1),
    ("to_jsonb", 1, 1),
    ("to_number", 2, 2),
    ("to_timestamp", 1, 2),
    ("translate", 3, 3),
    ("trunc", 1, 2),
    ("upper", 1, 1),
];

#[cfg(test)]
mod tests;
