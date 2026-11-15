//! Column equality fixes an allowlisted immutable expression of that column.
//! Recognized functions and cast families are listed in
//! `docs/rules/postgres-bounded-statements-proof.md`.
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::CatalogTable;
use sqlparser::ast::{
    CastKind, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, ObjectNamePart,
};

const FUNCTIONS: &[&str] = &["btrim", "lower", "upper"];

const FAMILIES: &[(&str, &[&str])] = &[
    (
        "text",
        &["char varying", "character varying", "text", "varchar"],
    ),
    (
        "integer",
        &[
            "bigint", "int", "int2", "int4", "int8", "integer", "smallint",
        ],
    ),
    ("uuid", &["uuid"]),
    ("boolean", &["bool", "boolean"]),
];

/// Catalog column fixed when the statement equates that column.
pub(super) fn source_column(expression: &Expr, table: &CatalogTable) -> Option<String> {
    let name = match unwrap_expr(expression) {
        Expr::Function(function) => function_column(function)?,
        Expr::Cast {
            kind: CastKind::Cast | CastKind::DoubleColon,
            expr,
            data_type,
            format: None,
        } => cast_column(expr, &data_type.to_string(), table)?,
        _ => return None,
    };
    column_type(table, &name)?;
    Some(name)
}

fn function_column(function: &Function) -> Option<String> {
    if !plain_call(function) {
        return None;
    }
    let name = builtin_function_name(function)?;
    if !FUNCTIONS.contains(&name.as_str()) {
        return None;
    }
    let FunctionArguments::List(arguments) = &function.args else {
        return None;
    };
    if arguments.args.len() != 1
        || arguments.duplicate_treatment.is_some()
        || !arguments.clauses.is_empty()
    {
        return None;
    }
    match arguments.args.first()? {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => bare_column(expr),
        _ => None,
    }
}

fn plain_call(function: &Function) -> bool {
    function.over.is_none()
        && function.filter.is_none()
        && function.null_treatment.is_none()
        && function.within_group.is_empty()
        && matches!(function.parameters, FunctionArguments::None)
}

fn builtin_function_name(function: &Function) -> Option<String> {
    let parts = function
        .name
        .0
        .iter()
        .map(ObjectNamePart::as_ident)
        .collect::<Option<Vec<_>>>()?;
    match parts.as_slice() {
        [name] if name.quote_style.is_none() => Some(ident_key(name)),
        [schema, name]
            if schema.quote_style.is_none()
                && name.quote_style.is_none()
                && ident_key(schema) == "pg_catalog" =>
        {
            Some(ident_key(name))
        }
        _ => None,
    }
}

fn cast_column(expr: &Expr, target: &str, table: &CatalogTable) -> Option<String> {
    let column = bare_column(expr)?;
    let source = column_type(table, &column)?;
    let source_family = family(source)?;
    (family(target)? == source_family).then_some(column)
}

fn bare_column(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident_key(ident)),
        _ => None,
    }
}

fn column_type<'a>(table: &'a CatalogTable, name: &str) -> Option<&'a str> {
    table
        .columns
        .iter()
        .find(|column| column.name == name)
        .map(|column| column.data_type.as_str())
}

fn family(raw: &str) -> Option<&'static str> {
    let name = builtin_type_name(raw)?;
    FAMILIES
        .iter()
        .find_map(|(family, names)| names.contains(&name.as_str()).then_some(*family))
}

fn builtin_type_name(raw: &str) -> Option<String> {
    let lowered = raw.trim().to_ascii_lowercase();
    let unqualified = lowered.strip_prefix("pg_catalog.").unwrap_or(&lowered);
    if unqualified.contains('.') || unqualified.contains('"') {
        return None;
    }
    let name = unqualified.split('(').next()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

#[cfg(test)]
#[path = "implied_tests.rs"]
mod tests;
