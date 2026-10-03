//! The built-in functions the bound facts reason about, by name.
use super::super::value::is_placeholder_ident;
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

mod set_returning;
use set_returning::SET_RETURNING;

/// Catalog SRFs that return one row and therefore preserve a pure aggregate's cap.
const FIXED_ONE_ROW: &[&str] = &["pg_stat_get_recovery_prefetch"];

/// Functions whose result row count follows caller-provided argument values.
#[rustfmt::skip]
const CALLER_SIZED: &[&str] = &[
    "unnest", "generate_series", "generate_subscripts", "json_array_elements",
    "json_array_elements_text", "jsonb_array_elements", "jsonb_array_elements_text",
    "json_each", "json_each_text", "jsonb_each", "jsonb_each_text", "json_object_keys",
    "jsonb_object_keys", "string_to_table", "regexp_split_to_table", "regexp_matches",
    "json_populate_recordset", "jsonb_populate_recordset", "json_to_recordset",
    "jsonb_to_recordset", "jsonb_path_query", "jsonb_path_query_tz", "aclexplode",
    "pg_options_to_table", "pg_mcv_list_items", "pg_snapshot_xip", "txid_snapshot_xip",
    "ts_parse", "ts_debug", "ts_token_type",
];

/// Whether `name` is bare or `pg_catalog`-qualified and its bare part is in `list`. A function
/// of any other schema that shares a name is an ordinary function, whatever it returns.
fn builtin(name: &ObjectName, list: &[&str]) -> bool {
    let schema = name
        .0
        .len()
        .checked_sub(2)
        .and_then(|index| name.0[index].as_ident());
    let in_scope = match name.0.len() {
        1 => true,
        2 => schema.is_some_and(|schema| ident_key(schema) == "pg_catalog"),
        _ => false,
    };
    in_scope && named(name, list)
}

fn named(name: &ObjectName, list: &[&str]) -> bool {
    object_name_ident(name).is_some_and(|ident| list.contains(&ident_key(ident).as_str()))
}

/// A built-in aggregate: the bare name, or `pg_catalog.<name>`. A function in any other schema
/// that happens to share a name is an ordinary function, called once per row.
pub(super) fn is_aggregate(name: &ObjectName) -> bool {
    builtin(name, AGGREGATES)
}

/// A catalog set-returning function must have a builtin schema identity.
/// A same-named user function has an unknown cardinality rather than the catalog contract.
pub(super) fn is_set_returning(name: &ObjectName) -> bool {
    builtin(name, SET_RETURNING) && !builtin(name, FIXED_ONE_ROW)
}

/// A table function is sized by the caller only when it is a set-returning built-in over
/// arguments the text provides; any other function can return rows from anywhere.
pub(super) fn function_kind(name: &ObjectName, args: &TableFunctionArgs) -> SqlBoundItemKind {
    // A nested function can obtain server data even when its visible arguments are fixed.
    let given = args.args.iter().all(|arg| match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
        | FunctionArg::Named {
            arg: FunctionArgExpr::Expr(expr),
            ..
        }
        | FunctionArg::ExprNamed {
            arg: FunctionArgExpr::Expr(expr),
            ..
        } => !input_depends_on_data(expr, true),
        _ => false,
    });
    if given && builtin(name, CALLER_SIZED) {
        SqlBoundItemKind::Other
    } else {
        SqlBoundItemKind::Opaque
    }
}

/// `unnest(…)` as a table factor is sized by its arrays, unless one is taken from a query or
/// from another FROM item (the table factor is then evaluated per row of that item).
pub(super) fn unnest_kind(arrays: &[Expr]) -> SqlBoundItemKind {
    if arrays.iter().any(depends_on_data) {
        SqlBoundItemKind::Opaque
    } else {
        SqlBoundItemKind::Other
    }
}

/// Whether `expr` holds a subquery or a column: values the statement text does not provide.
pub(super) fn depends_on_data(expr: &Expr) -> bool {
    input_depends_on_data(expr, false)
}

fn input_depends_on_data(expr: &Expr, reject_calls: bool) -> bool {
    struct Found(bool, bool);
    impl Visitor for Found {
        type Break = ();
        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.0 = true;
            ControlFlow::Break(())
        }
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            let column = match expr {
                Expr::Identifier(ident) => !is_placeholder_ident(&ident.value),
                Expr::CompoundIdentifier(_) => true,
                Expr::Function(_) if self.1 => true,
                _ => false,
            };
            if column {
                self.0 = true;
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
    }
    let mut found = Found(false, reject_calls);
    let _ = expr.visit(&mut found);
    found.0
}

#[cfg(test)]
mod tests;
