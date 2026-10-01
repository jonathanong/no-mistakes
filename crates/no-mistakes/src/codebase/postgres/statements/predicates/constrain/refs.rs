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
    let column = parts.last()?.value.clone();
    let qualifier = match parts.len() {
        2 => parts[0].value.clone(),
        3.. => parts[parts.len() - 2].value.clone(),
        _ => return None,
    };
    Some((qualifier, column))
}

pub(super) fn qualifier_matches(qualifier: &str, ctx: &Ctx<'_>) -> bool {
    let instance = &ctx.instances[ctx.index];
    if instance
        .alias
        .as_ref()
        .is_some_and(|alias| alias.eq_ignore_ascii_case(qualifier))
    {
        return true;
    }
    instance.table.eq_ignore_ascii_case(qualifier)
        && ctx
            .instances
            .iter()
            .filter(|other| other.table.eq_ignore_ascii_case(qualifier))
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
