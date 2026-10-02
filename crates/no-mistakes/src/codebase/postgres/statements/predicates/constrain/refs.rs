use super::Ctx;
use crate::codebase::postgres::idents::{unwrap_expr, visit_child_exprs};
use sqlparser::ast::{Expr, Ident};

pub(super) fn references(expr: &Expr, ctx: &Ctx<'_>) -> bool {
    let expr = unwrap_expr(expr);
    if node_references(expr, ctx) {
        return true;
    }
    let mut found = false;
    visit_child_exprs(expr, &mut |child| {
        found = found || references(child, ctx);
    });
    found
}

pub(super) fn qualifier_and_column(parts: &[Ident]) -> Option<(String, String)> {
    let column = crate::codebase::postgres::idents::ident_key(parts.last()?);
    let qualifier = match parts.len() {
        2 => crate::codebase::postgres::idents::ident_key(&parts[0]),
        3.. => crate::codebase::postgres::idents::ident_key(&parts[parts.len() - 2]),
        _ => return None,
    };
    Some((qualifier, column))
}

pub(super) fn qualifier_matches(qualifier: &str, ctx: &Ctx<'_>) -> bool {
    let instance = &ctx.instances[ctx.index];
    if instance
        .alias
        .as_ref()
        .is_some_and(|alias| alias == qualifier)
    {
        return true;
    }
    instance.alias.is_none()
        && instance.table.rsplit('.').next() == Some(qualifier)
        && ctx
            .instances
            .iter()
            .filter(|other| {
                other.alias.is_none() && other.table.rsplit('.').next() == Some(qualifier)
            })
            .count()
            == 1
}

fn node_references(expr: &Expr, ctx: &Ctx<'_>) -> bool {
    match expr {
        Expr::Identifier(_) => ctx.from_items <= 1,
        Expr::CompoundIdentifier(parts) => qualifier_and_column(parts)
            .is_some_and(|(qualifier, _)| qualifier_matches(&qualifier, ctx)),
        _ => false,
    }
}
