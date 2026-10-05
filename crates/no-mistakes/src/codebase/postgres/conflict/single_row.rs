//! Syntactic proofs that an `INSERT` source yields at most one row.
//!
//! Anything not proven stays "potentially multi-row": a set-returning function or an
//! unrecognized expression must never be mistaken for a single row.
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, LimitClause, Query, SetExpr, Value,
};

/// Scalar functions that return exactly one value per input row.
const SCALAR_FUNCTIONS: &[&str] = &[
    "lower",
    "upper",
    "coalesce",
    "nullif",
    "concat",
    "now",
    "current_timestamp",
    "current_date",
    "current_time",
    "localtimestamp",
    "clock_timestamp",
    "gen_random_uuid",
    "uuid_generate_v4",
    "jsonb_build_object",
    "json_build_object",
];

pub(super) fn query_is_potentially_multi_row(query: &Query, binds: &[(u32, u32)]) -> bool {
    !limits_to_one_row(query) && body_is_potentially_multi_row(query.body.as_ref(), binds)
}

fn body_is_potentially_multi_row(body: &SetExpr, binds: &[(u32, u32)]) -> bool {
    match body {
        SetExpr::Values(values) => values.rows.len() > 1,
        SetExpr::Insert(_) | SetExpr::Update(_) | SetExpr::Delete(_) | SetExpr::Merge(_) => true,
        SetExpr::Query(query) => query_is_potentially_multi_row(query, binds),
        // Without FROM a SELECT is one row, unless a set-returning function expands it.
        SetExpr::Select(select) => {
            !select.from.is_empty()
                || !select.projection.iter().all(|item| {
                    projection_expr(item).is_some_and(|expr| single_valued(expr, false, binds))
                })
        }
        SetExpr::SetOperation { .. } | SetExpr::Table(_) => true,
    }
}

fn limits_to_one_row(query: &Query) -> bool {
    let limit = match &query.limit_clause {
        Some(LimitClause::LimitOffset {
            limit: Some(limit),
            limit_by,
            ..
        }) if limit_by.is_empty() => Some(limit),
        _ => None,
    };
    limit.is_some_and(is_zero_or_one)
        || query.fetch.as_ref().is_some_and(|fetch| {
            !fetch.percent && fetch.quantity.as_ref().is_none_or(is_zero_or_one)
        })
}

fn is_zero_or_one(expr: &Expr) -> bool {
    matches!(expr, Expr::Value(value)
        if matches!(&value.value, Value::Number(number, _) if number == "0" || number == "1"))
}

pub(super) fn projection_expr(item: &sqlparser::ast::SelectItem) -> Option<&Expr> {
    match item {
        sqlparser::ast::SelectItem::UnnamedExpr(expr)
        | sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } => Some(expr),
        _ => None,
    }
}

/// An interpolation recovered from a template literal reaches the parser as a
/// `sql_placeholder_N` identifier; only one at a recovered position is a bound value, so a
/// user-authored identifier with that spelling is still a column.
pub(super) fn is_bind(ident: &sqlparser::ast::Ident, binds: &[(u32, u32)]) -> bool {
    crate::codebase::postgres::statements::value::is_placeholder_ident_at(ident, Some(binds))
}

/// Whether `expr` produces one value per input row (columns only when `allow_columns`).
pub(super) fn single_valued(expr: &Expr, allow_columns: bool, binds: &[(u32, u32)]) -> bool {
    match expr {
        Expr::Value(_) | Expr::Subquery(_) => true,
        Expr::Identifier(ident) => allow_columns || is_bind(ident, binds),
        Expr::CompoundIdentifier(_) => allow_columns,
        Expr::Nested(inner)
        | Expr::Cast { expr: inner, .. }
        | Expr::UnaryOp { expr: inner, .. } => single_valued(inner, allow_columns, binds),
        Expr::BinaryOp { left, right, .. } => {
            single_valued(left, allow_columns, binds) && single_valued(right, allow_columns, binds)
        }
        Expr::Function(function) => {
            let name = function.name.to_string().to_ascii_lowercase();
            function.over.is_none()
                && SCALAR_FUNCTIONS.contains(&name.as_str())
                && match &function.args {
                    FunctionArguments::None => true,
                    FunctionArguments::List(list) => list.args.iter().all(|arg| match arg {
                        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => {
                            single_valued(expr, allow_columns, binds)
                        }
                        _ => false,
                    }),
                    FunctionArguments::Subquery(_) => false,
                }
        }
        _ => false,
    }
}

pub(super) fn select_list(body: &SetExpr) -> Option<Vec<String>> {
    let SetExpr::Select(select) = body else {
        return None;
    };
    select
        .projection
        .iter()
        .map(|item| projection_expr(item).map(ToString::to_string))
        .collect()
}
