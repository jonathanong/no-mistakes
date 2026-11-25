//! Literal extraction helpers for PostgreSQL read-only setting evaluation.
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Query, SetExpr, Statement, Value,
};

pub(super) fn direct_write_kind(statement: &Statement) -> Option<&'static str> {
    match statement {
        Statement::Insert(_) => Some("INSERT"),
        Statement::Update(_) => Some("UPDATE"),
        Statement::Delete(_) => Some("DELETE"),
        Statement::Merge(_) => Some("MERGE"),
        Statement::Query(query) => query_write_kind(query),
        _ => None,
    }
}

fn query_write_kind(query: &Query) -> Option<&'static str> {
    match query.body.as_ref() {
        SetExpr::Insert(_) => Some("INSERT"),
        SetExpr::Update(_) => Some("UPDATE"),
        SetExpr::Delete(_) => Some("DELETE"),
        SetExpr::Merge(_) => Some("MERGE"),
        _ => query.with.as_ref().and_then(|with| {
            with.cte_tables
                .iter()
                .find_map(|cte| query_write_kind(&cte.query))
        }),
    }
}

pub(super) fn three_arg_exprs(args: &FunctionArguments) -> Option<(&Expr, &Expr, &Expr)> {
    match args {
        FunctionArguments::List(arguments) => match arguments.args.as_slice() {
            [a, b, c] => Some((arg_expr(a)?, arg_expr(b)?, arg_expr(c)?)),
            _ => None,
        },
        _ => None,
    }
}

fn arg_expr(arg: &FunctionArg) -> Option<&Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => Some(expr),
        _ => None,
    }
}

pub(super) fn unwrap_expr(mut expr: &Expr) -> &Expr {
    while let Expr::Nested(inner) | Expr::Cast { expr: inner, .. } = expr {
        expr = inner;
    }
    expr
}

pub(super) fn string_literal(expr: &Expr) -> Option<&str> {
    let Expr::Value(value) = unwrap_expr(expr) else {
        return None;
    };
    match &value.value {
        Value::SingleQuotedString(text) => Some(text.as_str()),
        _ => None,
    }
}

pub(super) fn parse_bool_guc(expr: &Expr) -> Option<bool> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::Boolean(flag) => Some(*flag),
            Value::SingleQuotedString(text) => parse_bool_text(text),
            _ => None,
        },
        Expr::Identifier(ident) => parse_bool_text(&ident.value),
        _ => None,
    }
}

fn parse_bool_text(text: &str) -> Option<bool> {
    match text.trim().to_ascii_lowercase().as_str() {
        "on" | "true" | "t" | "yes" | "y" | "1" => Some(true),
        "off" | "false" | "f" | "no" | "n" | "0" => Some(false),
        _ => None,
    }
}
