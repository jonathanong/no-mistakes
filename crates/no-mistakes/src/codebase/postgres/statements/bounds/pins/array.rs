//! Finite constructor leaves preserve source dependencies and scalar-type requirements.
use super::{merge, Resolver, Sourced};
use crate::codebase::postgres::idents::{ident_key, object_name_ident, unwrap_expr};
use crate::codebase::postgres::SqlPinSource;
use sqlparser::ast::{
    DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, UnaryOperator, Value,
};

mod fixed_boolean;
mod indexed;
mod scalar;
use fixed_boolean::fixed_scalar_boolean;
pub(super) use indexed::indexed_base;

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
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Option<Sourced> {
    fn columns(
        expr: &Expr,
        resolver: &Resolver,
        out: &mut Vec<(usize, String)>,
        indexed: &mut Vec<(usize, String)>,
        types: &mut Vec<String>,
        caller_only: bool,
        positions: super::super::super::value::PlaceholderPositions<'_>,
    ) -> Option<()> {
        match unwrap_expr(expr) {
            Expr::Value(_) | Expr::Interval(_) => Some(()),
            expr if fixed_scalar_boolean(expr) => Some(()),
            Expr::TypedString(literal) if !matches!(literal.data_type, DataType::Array(_)) => {
                // Parser-custom names such as XML still need catalog scalar evidence.
                if matches!(literal.data_type, DataType::Custom(_, _)) {
                    types.push(literal.data_type.to_string());
                }
                Some(())
            }
            Expr::UnaryOp { .. } if numeric_literal(expr) => Some(()),
            Expr::Identifier(ident)
                if super::super::super::value::is_placeholder_ident_at(ident, positions) =>
            {
                Some(())
            }
            Expr::Identifier(_) | Expr::CompoundIdentifier(_) => {
                if caller_only {
                    return None;
                }
                out.push(resolver.column(expr)?);
                Some(())
            }
            Expr::CompoundFieldAccess { root, access_chain } if !caller_only => {
                let (base, _) = indexed_base(root, access_chain)?;
                indexed.push(resolver.column(&base)?);
                Some(())
            }
            Expr::Cast {
                expr,
                data_type: DataType::Array(_),
                ..
            } => {
                // A scalar text column can decode an arbitrarily large array.
                if let Some(array) = constructor(expr) {
                    for element in &array.elem {
                        columns(
                            element,
                            resolver,
                            out,
                            indexed,
                            types,
                            caller_only,
                            positions,
                        )?;
                    }
                } else {
                    columns(expr, resolver, out, indexed, types, true, positions)?;
                }
                Some(())
            }
            Expr::Cast {
                expr,
                data_type: data_type @ DataType::Custom(_, _),
                ..
            } => {
                types.push(data_type.to_string());
                if caller_only {
                    columns(expr, resolver, out, indexed, types, true, positions)?;
                }
                Some(())
            }
            Expr::Cast {
                expr, data_type, ..
            } if !matches!(data_type, DataType::Array(_) | DataType::Custom(_, _)) => {
                if caller_only {
                    columns(expr, resolver, out, indexed, types, true, positions)?;
                }
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
                    columns(expr, resolver, out, indexed, types, caller_only, positions)?;
                }
                Some(())
            }
            Expr::Function(function) => {
                for expr in scalar::arguments(function)? {
                    columns(expr, resolver, out, indexed, types, caller_only, positions)?;
                }
                Some(())
            }
            Expr::Array(array) => {
                for element in &array.elem {
                    columns(
                        element,
                        resolver,
                        out,
                        indexed,
                        types,
                        caller_only,
                        positions,
                    )?;
                }
                Some(())
            }
            _ => None,
        }
    }
    let mut scalar_columns = Vec::new();
    let mut indexed_columns = Vec::new();
    let mut cast_types = Vec::new();
    let sources = elements
        .iter()
        .map(|element| {
            columns(
                element,
                resolver,
                &mut scalar_columns,
                &mut indexed_columns,
                &mut cast_types,
                false,
                positions,
            )?;
            resolver.source(element, pinned)
        })
        .collect::<Option<Vec<_>>>()?;
    let mut sourced = merge(sources);
    let items = match sourced.source {
        SqlPinSource::Items(items) => items,
        SqlPinSource::Value if !cast_types.is_empty() => Vec::new(),
        _ => return Some(sourced),
    };
    {
        scalar_columns.sort();
        scalar_columns.dedup();
        indexed_columns.sort();
        indexed_columns.dedup();
        cast_types.sort();
        cast_types.dedup();
        sourced.source = SqlPinSource::Array {
            items,
            scalar_columns,
            indexed_columns,
            cast_types,
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
