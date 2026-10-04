use super::scalar;
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{DataType, Expr, Function};

/// An array's size must follow caller input, not merely lack references to local rows.
pub(in super::super) fn caller_array(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        // The caller first establishes Value provenance through Resolver: these names
        // are binds or independently caller-owned outputs, never stored columns.
        Expr::Value(_) | Expr::Identifier(_) | Expr::CompoundIdentifier(_) => true,
        Expr::Cast {
            expr,
            data_type: DataType::Array(_),
            ..
        } => caller_array(expr),
        Expr::Function(function) => {
            array_arguments(function).is_some_and(|args| args.into_iter().all(caller_value))
        }
        _ => false,
    }
}

fn caller_value(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(_) | Expr::Interval(_) | Expr::Identifier(_) | Expr::CompoundIdentifier(_) => {
            true
        }
        Expr::Array(array) => array.elem.iter().all(caller_value),
        Expr::Cast {
            expr, data_type, ..
        } if !matches!(data_type, DataType::Custom(..)) => caller_value(expr),
        Expr::Function(function) => array_arguments(function)
            .or_else(|| scalar::arguments(function))
            .is_some_and(|args| args.into_iter().all(caller_value)),
        _ => false,
    }
}

fn array_arguments(function: &Function) -> Option<Vec<&Expr>> {
    if super::super::super::functions::conditional_form(function) && function.over.is_none() {
        return scalar::positional_arguments(function, 1, usize::MAX);
    }
    scalar::arguments_for(function, ARRAY_SIGNATURES)
}

// Each operation sizes its array from supplied values; none fetches server-owned rows.
const ARRAY_SIGNATURES: &[(&str, usize, usize)] = &[
    ("array_append", 2, 2),
    ("array_prepend", 2, 2),
    ("array_cat", 2, 2),
    ("array_remove", 2, 2),
    ("array_positions", 2, 2),
    ("trim_array", 2, 2),
    ("array_replace", 3, 3),
    ("array_fill", 2, 3),
    ("string_to_array", 2, 3),
    ("regexp_split_to_array", 2, 3),
];

/// Casts can make a subquery yield one array value rather than one key per row.
pub(in super::super) fn scalar_subquery(expr: &Expr) -> Option<&sqlparser::ast::Query> {
    match unwrap_expr(expr) {
        Expr::Subquery(query) => Some(query),
        Expr::Cast { expr, .. } => scalar_subquery(expr),
        _ => None,
    }
}
