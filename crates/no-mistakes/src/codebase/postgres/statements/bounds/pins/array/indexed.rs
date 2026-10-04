//! Canonical column roots for scalar array access chains.
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{AccessExpr, Expr, Subscript};

// sqlparser places `o.ids[1]` into a dotted access chain, not a compound column root.
pub(in super::super) fn indexed_base<'a>(
    root: &Expr,
    access_chain: &'a [AccessExpr],
) -> Option<(Expr, Vec<&'a Expr>)> {
    let mut parts = match unwrap_expr(root) {
        Expr::Identifier(ident) => vec![ident.clone()],
        Expr::CompoundIdentifier(parts) => parts.clone(),
        _ => return None,
    };
    let mut indexes = Vec::new();
    for access in access_chain {
        match access {
            AccessExpr::Dot(Expr::Identifier(ident)) if indexes.is_empty() => {
                parts.push(ident.clone())
            }
            AccessExpr::Subscript(Subscript::Index { index }) => indexes.push(index),
            _ => return None,
        }
    }
    if indexes.is_empty() {
        return None;
    }
    let base = if parts.len() == 1 {
        Expr::Identifier(parts.remove(0))
    } else {
        Expr::CompoundIdentifier(parts)
    };
    Some((base, indexes))
}
