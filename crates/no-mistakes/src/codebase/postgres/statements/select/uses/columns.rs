use super::super::super::value::{is_placeholder_ident_at, PlaceholderPositions};
use super::{peel, BaseRel};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{Expr, Ident};

// Ordinary SQL keeps marker-shaped column names; only recovered positions identify binds.
pub(super) fn is_recovered_bind(ident: &Ident, positions: PlaceholderPositions<'_>) -> bool {
    positions.is_some() && is_placeholder_ident_at(ident, positions)
}

pub(super) fn bare(
    expr: &Expr,
    positions: PlaceholderPositions<'_>,
) -> Option<(Option<String>, String)> {
    match peel(expr) {
        Expr::Identifier(ident) if !is_recovered_bind(ident, positions) => {
            Some((None, ident_key(ident)))
        }
        Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
            let last = parts.last()?;
            if is_recovered_bind(last, positions) {
                return None;
            }
            let column = ident_key(last);
            let qualifier = parts[..parts.len() - 1]
                .iter()
                .map(ident_key)
                .collect::<Vec<_>>()
                .join(".");
            Some((Some(qualifier), column))
        }
        _ => None,
    }
}

pub(super) fn resolve(rels: &[BaseRel], qualifier: Option<&str>) -> Option<String> {
    let Some(qualifier) = qualifier else {
        return if rels.len() == 1 && !rels[0].unknown {
            Some(rels[0].table.clone())
        } else {
            Some(String::new())
        };
    };
    rels.iter()
        .find(|rel| rel.alias.as_deref() == Some(qualifier))
        .or_else(|| {
            rels.iter().find(|rel| {
                rel.table == qualifier || rel.table.rsplit('.').next() == Some(qualifier)
            })
        })
        .map(|rel| rel.table.clone())
}
