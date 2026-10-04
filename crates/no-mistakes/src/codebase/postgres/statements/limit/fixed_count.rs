use super::super::value;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Value};

/// A count the statement text or its caller decides: a literal, a bind (`$1`, or an
/// interpolation recovered from a template literal) or an expression of them. NULL means no
/// limit, and a subquery or a column can return any number.
pub(super) fn is_fixed_at(expr: &Expr, positions: value::PlaceholderPositions<'_>) -> bool {
    matches!(
        fixed_count_at(expr, positions),
        Some(FixedCount::Caller | FixedCount::NonNull)
    )
}

#[derive(Clone, Copy, PartialEq)]
enum FixedCount {
    Nullable,
    Caller,
    NonNull,
}

/// A bind retains its caller contract, but cannot guarantee a non-NULL conditional fallback.
fn fixed_count_at(expr: &Expr, positions: value::PlaceholderPositions<'_>) -> Option<FixedCount> {
    match expr {
        Expr::Nested(inner) => fixed_count_at(inner, positions),
        Expr::Value(value) => Some(match value.value {
            Value::Null => FixedCount::Nullable,
            Value::Placeholder(_) => FixedCount::Caller,
            _ => FixedCount::NonNull,
        }),
        Expr::Identifier(ident) => {
            value::is_placeholder_ident_at(ident, positions).then_some(FixedCount::Caller)
        }
        Expr::Cast { expr, .. } | Expr::UnaryOp { expr, .. } => {
            is_fixed_at(expr, positions).then_some(FixedCount::Caller)
        }
        Expr::BinaryOp { left, right, .. } => (is_fixed_at(left, positions)
            && is_fixed_at(right, positions))
        .then_some(FixedCount::Caller),
        // Only the functions that return NULL for nothing but NULL arguments: `NULLIF(1, 1)`,
        // like any function that can produce NULL from fixed inputs, is `LIMIT ALL`.
        Expr::Function(function) => {
            let pick = function.name.0.len() == 1
                && object_name_ident(&function.name).is_some_and(|ident| {
                    ident.quote_style.is_none()
                        && ["coalesce", "least", "greatest"].contains(&ident_key(ident).as_str())
                });
            if !pick {
                return None;
            }
            match &function.args {
                FunctionArguments::List(list) => {
                    let (nullable, non_null) = list.args.iter().try_fold(
                        (false, false),
                        |(nullable, non_null), arg| match arg {
                            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => {
                                // Visit every argument once, including those after a fixed fallback:
                                // unknown/data-derived inputs never gain a caller-owned count proof.
                                let count = fixed_count_at(expr, positions)?;
                                Some((
                                    nullable || count == FixedCount::Nullable,
                                    non_null || count == FixedCount::NonNull,
                                ))
                            }
                            _ => None,
                        },
                    )?;
                    Some(if non_null {
                        FixedCount::NonNull
                    } else if nullable {
                        FixedCount::Nullable
                    } else {
                        FixedCount::Caller
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}
