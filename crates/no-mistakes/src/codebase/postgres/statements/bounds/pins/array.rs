//! Finite constructor leaves preserve source dependencies and scalar-type requirements.
use super::{merge, Resolver, Sourced};
use crate::codebase::postgres::idents::{ident_key, object_name_ident, unwrap_expr};
use crate::codebase::postgres::SqlPinSource;
use sqlparser::ast::{
    DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, UnaryOperator, Value,
};

pub(super) fn constructor(expr: &Expr) -> Option<&sqlparser::ast::Array> {
    match unwrap_expr(expr) {
        Expr::Array(array) => Some(array),
        Expr::Cast {
            expr,
            data_type: DataType::Array(_),
            ..
        } => constructor(expr),
        _ => None,
    }
}

/// Literal/bind leaves are caller-sized; a column leaf requires catalog scalar evidence.
/// Nested constructors flatten, so their leaves need the same proof rather than an arity guess.
pub(super) fn finite_array(
    elements: &[Expr],
    pinned: usize,
    resolver: &Resolver,
) -> Option<Sourced> {
    fn columns(expr: &Expr, resolver: &Resolver, out: &mut Vec<(usize, String)>) -> Option<()> {
        match unwrap_expr(expr) {
            Expr::Value(_) => Some(()),
            Expr::UnaryOp { .. } if numeric_literal(expr) => Some(()),
            Expr::Identifier(ident)
                if super::super::super::value::is_placeholder_ident(&ident.value) =>
            {
                Some(())
            }
            Expr::Identifier(_) | Expr::CompoundIdentifier(_) => {
                out.push(resolver.column(expr)?);
                Some(())
            }
            Expr::Cast {
                expr,
                data_type: DataType::Array(_),
                ..
            } => columns(expr, resolver, out),
            Expr::Cast { data_type, .. }
                if !matches!(data_type, DataType::Array(_) | DataType::Custom(_, _)) =>
            {
                Some(())
            }
            Expr::Function(function)
                if function.name.0.len() == 1
                    && object_name_ident(&function.name).is_some_and(|name| {
                        name.quote_style.is_none()
                            && ["coalesce", "least", "greatest"].contains(&ident_key(name).as_str())
                    }) =>
            {
                let FunctionArguments::List(arguments) = &function.args else {
                    return None;
                };
                for argument in &arguments.args {
                    let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = argument else {
                        return None;
                    };
                    columns(expr, resolver, out)?;
                }
                Some(())
            }
            Expr::Array(array) => {
                for element in &array.elem {
                    columns(element, resolver, out)?;
                }
                Some(())
            }
            _ => None,
        }
    }
    let mut scalar_columns = Vec::new();
    let sources = elements
        .iter()
        .map(|element| {
            columns(element, resolver, &mut scalar_columns)?;
            resolver.source(element, pinned)
        })
        .collect::<Option<Vec<_>>>()?;
    let mut sourced = merge(sources);
    if let SqlPinSource::Items(items) = sourced.source {
        scalar_columns.sort();
        scalar_columns.dedup();
        sourced.source = SqlPinSource::Array {
            items,
            scalar_columns,
        };
    }
    Some(sourced)
}

// Numeric signs do not change scalar cardinality; arbitrary overloaded operators need proof.
fn numeric_literal(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(&value.value, Value::Number(..)),
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } => numeric_literal(expr),
        _ => false,
    }
}
