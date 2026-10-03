//! The built-in functions the bound facts reason about, by name.
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use crate::codebase::postgres::statements::SqlBoundItemKind;
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, ObjectName, Query, TableFunctionArgs, Visit, Visitor,
};
use std::ops::ControlFlow;

/// The built-in aggregates, ordered-set and hypothetical-set aggregates included.
#[rustfmt::skip]
const AGGREGATES: &[&str] = &[
    "count", "sum", "avg", "min", "max", "bool_and", "bool_or", "every", "array_agg",
    "string_agg", "json_agg", "jsonb_agg", "json_object_agg", "jsonb_object_agg",
    "json_objectagg", "json_arrayagg", "xmlagg", "bit_and", "bit_or", "bit_xor", "stddev",
    "stddev_pop", "stddev_samp", "variance", "var_pop", "var_samp", "corr", "covar_pop",
    "covar_samp", "regr_avgx", "regr_avgy", "regr_count", "regr_intercept", "regr_r2",
    "regr_slope", "regr_sxx", "regr_sxy", "regr_syy", "percentile_cont", "percentile_disc",
    "mode", "rank", "dense_rank", "percent_rank", "cume_dist", "any_value", "range_agg",
    "range_intersect_agg",
];

/// Set-returning functions whose row count follows the value of their arguments. In FROM, with
/// arguments the statement or its caller supplies, they are sized by the caller; in a select
/// list they turn one aggregate row into many.
#[rustfmt::skip]
const SET_RETURNING: &[&str] = &[
    "unnest", "generate_series", "generate_subscripts", "json_array_elements",
    "json_array_elements_text", "jsonb_array_elements", "jsonb_array_elements_text",
    "json_each", "json_each_text", "jsonb_each", "jsonb_each_text", "json_object_keys",
    "jsonb_object_keys", "string_to_table", "regexp_split_to_table", "regexp_matches",
    "json_populate_recordset", "jsonb_populate_recordset", "json_to_recordset",
    "jsonb_to_recordset",
];

fn named(name: &ObjectName, list: &[&str]) -> Option<bool> {
    let ident = object_name_ident(name)?;
    Some(list.contains(&ident_key(ident).as_str()))
}

/// A built-in aggregate: the bare name, or `pg_catalog.<name>`. A function in any other schema
/// that happens to share a name is an ordinary function, called once per row.
pub(super) fn is_aggregate(name: &ObjectName) -> bool {
    let schema = name
        .0
        .len()
        .checked_sub(2)
        .and_then(|index| name.0[index].as_ident());
    let builtin = match name.0.len() {
        1 => true,
        2 => schema.is_some_and(|schema| ident_key(schema) == "pg_catalog"),
        _ => false,
    };
    builtin && named(name, AGGREGATES) == Some(true)
}

/// A set-returning function by name, whichever schema it is called through.
pub(super) fn is_set_returning(name: &ObjectName) -> bool {
    named(name, SET_RETURNING) == Some(true)
}

/// A table function is sized by the caller only when it is a set-returning built-in over
/// arguments the text provides; any other function can return rows from anywhere.
pub(super) fn function_kind(name: &ObjectName, args: &TableFunctionArgs) -> SqlBoundItemKind {
    let sized = args.args.iter().all(|arg| match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
        | FunctionArg::Named {
            arg: FunctionArgExpr::Expr(expr),
            ..
        } => !contains_query(expr),
        _ => true,
    });
    if sized && is_set_returning(name) {
        SqlBoundItemKind::Other
    } else {
        SqlBoundItemKind::Opaque
    }
}

/// `unnest(…)` as a table factor is sized by its arrays, unless one is taken from a query.
pub(super) fn unnest_kind(arrays: &[Expr]) -> SqlBoundItemKind {
    if arrays.iter().any(contains_query) {
        SqlBoundItemKind::Opaque
    } else {
        SqlBoundItemKind::Other
    }
}

/// Whether `expr` holds a subquery, whose rows the statement text does not size.
pub(super) fn contains_query(expr: &Expr) -> bool {
    struct Found(bool);
    impl Visitor for Found {
        type Break = ();
        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.0 = true;
            ControlFlow::Break(())
        }
    }
    let mut found = Found(false);
    let _ = expr.visit(&mut found);
    found.0
}
