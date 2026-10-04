//! Finite constructor leaves preserve source dependencies and scalar-type requirements.
use super::{merge, Resolver, Sourced};
use crate::codebase::postgres::idents::{ident_key, object_name_ident, unwrap_expr};
use crate::codebase::postgres::SqlPinSource;
use sqlparser::ast::{
    DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, UnaryOperator, Value,
};

mod indexed;
mod scalar;
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
) -> Option<Sourced> {
    fn columns(
        expr: &Expr,
        resolver: &Resolver,
        out: &mut Vec<(usize, String)>,
        indexed: &mut Vec<(usize, String)>,
        types: &mut Vec<String>,
        caller_only: bool,
    ) -> Option<()> {
        match unwrap_expr(expr) {
            Expr::Value(_) | Expr::Interval(_) => Some(()),
            Expr::IsNull(_)
            | Expr::IsNotNull(_)
            | Expr::IsTrue(_)
            | Expr::IsNotTrue(_)
            | Expr::IsFalse(_)
            | Expr::IsNotFalse(_)
            | Expr::IsUnknown(_)
            | Expr::IsNotUnknown(_)
            | Expr::IsDistinctFrom(_, _)
            | Expr::IsNotDistinctFrom(_, _)
                if fixed_scalar_boolean(expr) =>
            {
                Some(())
            }
            Expr::UnaryOp {
                op: UnaryOperator::Not,
                ..
            }
            | Expr::BinaryOp {
                op:
                    sqlparser::ast::BinaryOperator::And
                    | sqlparser::ast::BinaryOperator::Or
                    | sqlparser::ast::BinaryOperator::Xor,
                ..
            } if fixed_scalar_boolean(expr) => Some(()),
            Expr::TypedString(literal) if !matches!(literal.data_type, DataType::Array(_)) => {
                // Parser-custom names such as XML still need catalog scalar evidence.
                if matches!(literal.data_type, DataType::Custom(_, _)) {
                    types.push(literal.data_type.to_string());
                }
                Some(())
            }
            Expr::UnaryOp { .. } if numeric_literal(expr) => Some(()),
            Expr::Identifier(ident)
                if super::super::super::value::is_placeholder_ident(&ident.value) =>
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
                        columns(element, resolver, out, indexed, types, caller_only)?;
                    }
                } else {
                    columns(expr, resolver, out, indexed, types, true)?;
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
                    columns(expr, resolver, out, indexed, types, true)?;
                }
                Some(())
            }
            Expr::Cast {
                expr, data_type, ..
            } if !matches!(data_type, DataType::Array(_) | DataType::Custom(_, _)) => {
                if caller_only {
                    columns(expr, resolver, out, indexed, types, true)?;
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
                    columns(expr, resolver, out, indexed, types, caller_only)?;
                }
                Some(())
            }
            Expr::Function(function) => {
                for expr in scalar::arguments(function)? {
                    columns(expr, resolver, out, indexed, types, caller_only)?;
                }
                Some(())
            }
            Expr::Array(array) => {
                for element in &array.elem {
                    columns(element, resolver, out, indexed, types, caller_only)?;
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

/// A boolean expression made only from scalar literals cannot expand an array's row count.
/// Calls and column references stay opaque: even a familiar function name may be overridden.
fn fixed_scalar_boolean(expr: &Expr) -> bool {
    use sqlparser::ast::{BinaryOperator, Expr as SqlExpr};

    match unwrap_expr(expr) {
        SqlExpr::Value(value) if matches!(value.value, Value::Boolean(_) | Value::Null) => true,
        SqlExpr::IsNull(inner)
        | SqlExpr::IsNotNull(inner)
        | SqlExpr::IsTrue(inner)
        | SqlExpr::IsNotTrue(inner)
        | SqlExpr::IsFalse(inner)
        | SqlExpr::IsNotFalse(inner)
        | SqlExpr::IsUnknown(inner)
        | SqlExpr::IsNotUnknown(inner) => fixed_scalar_value(inner),
        SqlExpr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => fixed_scalar_boolean(expr),
        SqlExpr::BinaryOp {
            left,
            op: BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor,
            right,
        } => fixed_scalar_boolean(left) && fixed_scalar_boolean(right),
        SqlExpr::BinaryOp {
            left,
            op:
                BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq,
            right,
        }
        | SqlExpr::IsDistinctFrom(left, right)
        | SqlExpr::IsNotDistinctFrom(left, right) => {
            fixed_scalar_value(left) && fixed_scalar_value(right)
        }
        _ => false,
    }
}

fn fixed_scalar_value(expr: &Expr) -> bool {
    use sqlparser::ast::Expr as SqlExpr;

    match unwrap_expr(expr) {
        SqlExpr::Value(_) | SqlExpr::Interval(_) | SqlExpr::TypedString(_) => true,
        SqlExpr::Identifier(ident)
            if super::super::super::value::is_placeholder_ident(&ident.value) =>
        {
            true
        }
        SqlExpr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } => fixed_scalar_value(expr),
        SqlExpr::UnaryOp {
            op: UnaryOperator::Not,
            ..
        }
        | SqlExpr::BinaryOp {
            op:
                sqlparser::ast::BinaryOperator::And
                | sqlparser::ast::BinaryOperator::Or
                | sqlparser::ast::BinaryOperator::Xor
                | sqlparser::ast::BinaryOperator::Eq
                | sqlparser::ast::BinaryOperator::NotEq
                | sqlparser::ast::BinaryOperator::Gt
                | sqlparser::ast::BinaryOperator::GtEq
                | sqlparser::ast::BinaryOperator::Lt
                | sqlparser::ast::BinaryOperator::LtEq,
            ..
        }
        | SqlExpr::IsNull(_)
        | SqlExpr::IsNotNull(_)
        | SqlExpr::IsTrue(_)
        | SqlExpr::IsNotTrue(_)
        | SqlExpr::IsFalse(_)
        | SqlExpr::IsNotFalse(_)
        | SqlExpr::IsUnknown(_)
        | SqlExpr::IsNotUnknown(_)
        | SqlExpr::IsDistinctFrom(_, _)
        | SqlExpr::IsNotDistinctFrom(_, _) => fixed_scalar_boolean(expr),
        SqlExpr::BinaryOp { left, right, .. } => {
            fixed_scalar_value(left) && fixed_scalar_value(right)
        }
        SqlExpr::Cast { expr, .. } => fixed_scalar_value(expr),
        _ => false,
    }
}
