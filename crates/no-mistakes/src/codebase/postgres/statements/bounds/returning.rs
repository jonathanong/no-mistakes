//! Project a data-modifying CTE's RETURNING cardinality back to its target.
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{SqlBoundInputMode, SqlBoundOutput, SqlBoundQuery};
use sqlparser::ast::{Expr, SelectItem};

pub(super) fn project(mut query: SqlBoundQuery, returning: &[SelectItem]) -> Option<SqlBoundQuery> {
    if query.items.is_empty() {
        return None;
    }
    query.input_mode = SqlBoundInputMode::Returning;
    query.outputs = returning
        .iter()
        .map(|item| match item {
            SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(_, _) => None,
            _ => Some(SqlBoundOutput {
                name: returning_name(item),
                caller_sized: false,
                scalar: false,
                source_column: None,
                array_aggregate_column: None,
                constant_null: false,
                recursive_domain: false,
            }),
        })
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default();
    Some(query)
}

pub(super) fn name_returning(query: &mut SqlBoundQuery, returning: &[SelectItem]) {
    if query.outputs.len() != returning.len() {
        return;
    }
    for (output, item) in query.outputs.iter_mut().zip(returning) {
        if output.name.is_some() {
            continue;
        }
        if let Some(name) = returning_name(item) {
            output.name = Some(name);
        }
    }
}

pub(super) fn returning_name(item: &SelectItem) -> Option<String> {
    match item {
        SelectItem::ExprWithAlias { alias, .. } => Some(ident_key(alias)),
        SelectItem::UnnamedExpr(expr) => match unwrap_expr(expr) {
            Expr::Identifier(ident) => Some(ident_key(ident)),
            Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
            _ => None,
        },
        _ => None,
    }
}
