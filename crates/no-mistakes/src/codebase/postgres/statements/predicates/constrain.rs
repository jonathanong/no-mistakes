mod refs;
mod sets;

#[cfg(test)]
mod tests;

use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr};
use std::collections::BTreeSet;

pub(super) struct Instance<'a> {
    pub table: String,
    pub alias: Option<String>,
    pub joined: bool,
    pub on: Option<&'a Expr>,
}

#[derive(Default)]
struct Cols {
    proven: BTreeSet<String>,
    pending: BTreeSet<String>,
}

struct Ctx<'a> {
    instances: &'a [Instance<'a>],
    index: usize,
    from_items: usize,
    all_base: bool,
}

pub(super) fn constrain(
    instances: &[Instance<'_>],
    selection: Option<&Expr>,
    from_items: usize,
    all_base: bool,
) -> Vec<(Vec<String>, Vec<String>)> {
    instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            let ctx = Ctx {
                instances,
                index,
                from_items,
                all_base,
            };
            let mut cols = Cols::default();
            if instance.joined {
                if let Some(on) = instance.on {
                    cols = sets::union(cols, columns_from_expr(on, &ctx));
                }
            }
            if let Some(selection) = selection {
                cols = sets::union(cols, columns_from_expr(selection, &ctx));
            }
            (
                cols.proven.into_iter().collect(),
                cols.pending.into_iter().collect(),
            )
        })
        .collect()
}

fn columns_from_expr(expr: &Expr, ctx: &Ctx<'_>) -> Cols {
    match unwrap_expr(expr) {
        Expr::BinaryOp { left, op, right } => match op {
            BinaryOperator::And => {
                sets::union(columns_from_expr(left, ctx), columns_from_expr(right, ctx))
            }
            BinaryOperator::Or => {
                sets::intersect(columns_from_expr(left, ctx), columns_from_expr(right, ctx))
            }
            BinaryOperator::Eq
            | BinaryOperator::Gt
            | BinaryOperator::Lt
            | BinaryOperator::GtEq
            | BinaryOperator::LtEq => compare(left, right, ctx),
            _ => Cols::default(),
        },
        Expr::Between {
            expr,
            negated: false,
            low,
            high,
        } if !refs::references(low, ctx) && !refs::references(high, ctx) => {
            from_hit(column_hit(expr, ctx))
        }
        Expr::InList {
            expr,
            negated: false,
            ..
        }
        | Expr::InSubquery {
            expr,
            negated: false,
            ..
        } => from_hit(column_hit(expr, ctx)),
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } if !refs::references(right, ctx) => from_hit(column_hit(left, ctx)),
        _ => Cols::default(),
    }
}

fn compare(left: &Expr, right: &Expr, ctx: &Ctx<'_>) -> Cols {
    let mut cols = Cols::default();
    absorb(&mut cols, left, right, ctx);
    absorb(&mut cols, right, left, ctx);
    cols
}

fn absorb(cols: &mut Cols, column: &Expr, other: &Expr, ctx: &Ctx<'_>) {
    if refs::references(other, ctx) {
        return;
    }
    match column_hit(column, ctx) {
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

struct Hit {
    column: String,
    proven: bool,
}

fn column_hit(expr: &Expr, ctx: &Ctx<'_>) -> Option<Hit> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => {
            let column = ident.value.to_ascii_lowercase();
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
            refs::qualifier_matches(&qualifier, ctx).then(|| Hit {
                column: column.to_ascii_lowercase(),
                proven: true,
            })
        }
        _ => None,
    }
}

fn from_hit(hit: Option<Hit>) -> Cols {
    let mut cols = Cols::default();
    if let Some(hit) = hit {
        if hit.proven {
            cols.proven.insert(hit.column);
        } else {
            cols.pending.insert(hit.column);
        }
    }
    cols
}
