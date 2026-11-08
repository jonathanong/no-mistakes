use super::*;
pub(super) fn absorb(
    cols: &mut Cols,
    predicate: &Expr,
    column: &Expr,
    other: &Expr,
    ctx: &Ctx<'_, impl super::super::Projection>,
) {
    if refs::references(other, ctx) {
        return;
    }
    match project_hit(predicate, Some(other), column_hit(column, ctx), ctx) {
        Some(name) if name.proven => {
            cols.pending.remove(&name.column);
            cols.proven.insert(name.column);
        }
        Some(name) if !cols.proven.contains(&name.column) => {
            cols.pending.insert(name.column);
        }
        _ => {}
    }
}

pub(super) struct Hit {
    column: String,
    proven: bool,
}

pub(super) fn column_hit(expr: &Expr, ctx: &Ctx<'_, impl super::super::Projection>) -> Option<Hit> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident)
            if ctx.positions.is_some_and(|positions| {
                super::super::super::value::is_placeholder_ident_at(ident, Some(positions))
            }) =>
        {
            None
        }
        Expr::Identifier(ident) => {
            let column = crate::codebase::postgres::idents::ident_key(ident);
            if ctx.from_items <= 1 {
                Some(Hit {
                    column,
                    proven: true,
                })
            } else if ctx.all_base {
                Some(Hit {
                    column,
                    proven: false,
                })
            } else {
                None
            }
        }
        Expr::CompoundIdentifier(parts) => {
            let (qualifier, column) = refs::qualifier_and_column(parts)?;
            refs::qualifier_matches(&qualifier, ctx).then_some(Hit {
                column,
                proven: true,
            })
        }
        _ => None,
    }
}

pub(super) fn from_hit(
    predicate: &Expr,
    other: Option<&Expr>,
    hit: Option<Hit>,
    ctx: &Ctx<'_, impl super::super::Projection>,
) -> Cols {
    let mut cols = Cols::default();
    if let Some(hit) = project_hit(predicate, other, hit, ctx) {
        if hit.proven {
            cols.proven.insert(hit.column);
        } else {
            cols.pending.insert(hit.column);
        }
    }
    cols
}

fn project_hit(
    predicate: &Expr,
    other: Option<&Expr>,
    hit: Option<Hit>,
    ctx: &Ctx<'_, impl super::super::Projection>,
) -> Option<Hit> {
    let hit = hit?;
    let peers = ctx
        .instances
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != ctx.index)
        .map(|(_, instance)| instance.table.as_str());
    let (column, proven) = ctx.projection.map(
        predicate,
        other,
        &ctx.instances[ctx.index].table,
        hit.column,
        hit.proven,
        peers,
    )?;
    Some(Hit { column, proven })
}
