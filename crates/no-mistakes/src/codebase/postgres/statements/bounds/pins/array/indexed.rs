//! Canonical column roots for scalar array access chains.
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{AccessExpr, Expr, Subscript};

// sqlparser places `o.ids[1]` into a dotted access chain, not a compound column root.
pub(in super::super) fn indexed_base<'a>(
    root: &Expr,
    access_chain: &'a [AccessExpr],
) -> Option<(Expr, Vec<&'a Expr>)> {
    access_base(root, access_chain, false)
}

/// Slices retain their source row even though their result is not a scalar leaf.
pub(in super::super) fn argument_base<'a>(
    root: &Expr,
    access_chain: &'a [AccessExpr],
) -> Option<(Expr, Vec<&'a Expr>)> {
    access_base(root, access_chain, true)
}

fn access_base<'a>(
    root: &Expr,
    access_chain: &'a [AccessExpr],
    allow_slices: bool,
) -> Option<(Expr, Vec<&'a Expr>)> {
    let mut parts = match unwrap_expr(root) {
        Expr::Identifier(ident) => vec![ident.clone()],
        Expr::CompoundIdentifier(parts) => parts.clone(),
        _ => return None,
    };
    let mut indexes = Vec::new();
    let mut subscript = false;
    for access in access_chain {
        match access {
            AccessExpr::Dot(Expr::Identifier(ident)) if !subscript => parts.push(ident.clone()),
            AccessExpr::Subscript(Subscript::Index { index }) => {
                subscript = true;
                indexes.push(index);
            }
            AccessExpr::Subscript(Subscript::Slice {
                lower_bound,
                upper_bound,
                stride,
            }) if allow_slices => {
                subscript = true;
                indexes.extend(
                    [lower_bound.as_ref(), upper_bound.as_ref(), stride.as_ref()]
                        .into_iter()
                        .flatten(),
                );
            }
            _ => return None,
        }
    }
    if !subscript {
        return None;
    }
    let base = if parts.len() == 1 {
        Expr::Identifier(parts.remove(0))
    } else {
        Expr::CompoundIdentifier(parts)
    };
    Some((base, indexes))
}
