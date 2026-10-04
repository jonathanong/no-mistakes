use super::super::super::value::{is_placeholder_ident_at, PlaceholderPositions};
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::SqlBoundOutput;
use sqlparser::ast::{DataType, Expr, Select, SelectItem};

pub(super) fn select(select: &Select, positions: PlaceholderPositions<'_>) -> Vec<SqlBoundOutput> {
    select
        .projection
        .iter()
        .map(|item| match item {
            SelectItem::ExprWithAlias { expr, alias } => Some(SqlBoundOutput {
                name: Some(ident_key(alias)),
                caller_sized: caller_value(expr, positions),
            }),
            SelectItem::UnnamedExpr(expr) => Some(SqlBoundOutput {
                name: if let Expr::Identifier(ident) = expr {
                    Some(ident_key(ident))
                } else {
                    None
                },
                caller_sized: caller_value(expr, positions),
            }),
            // Wildcards have catalog-dependent width, so later ordinal aliases cannot be mapped.
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

// A custom cast or function may vary for each duplicate source row. Only direct values and
// trusted builtin scalar casts provide output proof; ordinary row bounds remain separate.
fn caller_value(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match expr {
        Expr::Value(_) => true,
        Expr::Identifier(ident) => is_placeholder_ident_at(ident, positions),
        Expr::Nested(expr) => caller_value(expr, positions),
        Expr::Cast {
            expr,
            data_type:
                DataType::Uuid
                | DataType::Text
                | DataType::Boolean
                | DataType::Bool
                | DataType::Int(_)
                | DataType::Integer(_)
                | DataType::BigInt(_)
                | DataType::SmallInt(_)
                | DataType::Numeric(_)
                | DataType::Decimal(_),
            ..
        } => caller_value(expr, positions),
        _ => false,
    }
}

pub(super) fn common(left: &[SqlBoundOutput], right: &[SqlBoundOutput]) -> Vec<SqlBoundOutput> {
    if left.len() != right.len() {
        return Vec::new();
    }
    left.iter()
        .zip(right)
        .map(|(left, right)| SqlBoundOutput {
            name: left.name.clone(),
            caller_sized: left.caller_sized && right.caller_sized,
        })
        .collect()
}
