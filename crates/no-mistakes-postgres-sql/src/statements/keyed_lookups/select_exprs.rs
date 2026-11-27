use super::select_item_expr;
use super::windows::referenced_expressions;
use crate::fx::FxHashSet;
use crate::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{
    CastKind, Distinct, Expr, GroupByExpr, Ident, Select, SelectItem, VisitMut, VisitorMut,
};
use std::cell::OnceCell;
use std::ops::ControlFlow;

#[derive(Default)]
pub(super) struct ProjectionTargets<'a> {
    projection: &'a [SelectItem],
    keys: OnceCell<FxHashSet<Expr>>,
}

impl<'a> ProjectionTargets<'a> {
    pub(super) fn new(projection: &'a [SelectItem]) -> Self {
        Self {
            projection,
            keys: OnceCell::new(),
        }
    }

    fn contains(&self, key: &Expr) -> bool {
        self.keys
            .get_or_init(|| {
                self.projection
                    .iter()
                    .filter_map(select_item_expr)
                    .map(target_identity)
                    .collect()
            })
            .contains(key)
    }

    pub(super) fn reuses(&self, expr: &Expr) -> bool {
        !self.projection.is_empty() && self.contains(&target_identity(expr))
    }
}

pub(super) fn inspectable<'a>(
    select: &'a Select,
    order_by: &'a [&'a Expr],
    projection_targets: &'a ProjectionTargets<'a>,
) -> impl Iterator<Item = &'a Expr> {
    let distinct_on = match &select.distinct {
        Some(Distinct::On(expressions)) => expressions.as_slice(),
        _ => &[],
    };
    let group_by = match &select.group_by {
        GroupByExpr::Expressions(expressions, _) => expressions.as_slice(),
        GroupByExpr::All(_) => &[],
    };

    let projection = select.projection.iter().filter_map(select_item_expr);
    let mut targets = FxHashSet::default();

    let clause_target = |expr| (expr, true);
    let independent = |expr| (expr, false);
    distinct_on
        .iter()
        .map(clause_target)
        .chain(projection.map(independent))
        .chain(select.selection.iter().map(independent))
        .chain(group_by.iter().map(clause_target))
        .chain(select.having.iter().map(independent))
        .chain(referenced_expressions(select, order_by))
        .chain(order_by.iter().copied().map(clause_target))
        .filter_map(move |(expr, can_reuse)| {
            if can_reuse {
                let key = target_identity(expr);
                if projection_targets.contains(&key) || !targets.insert(key) {
                    return None;
                }
            }
            Some(expr)
        })
}

fn target_identity(expr: &Expr) -> Expr {
    struct Identifiers;
    impl VisitorMut for Identifiers {
        type Break = ();

        fn pre_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
            if matches!(expr, Expr::Nested(_)) {
                *expr = unwrap_expr(expr).clone();
            }
            if let Expr::Cast {
                kind: kind @ CastKind::DoubleColon,
                ..
            } = expr
            {
                *kind = CastKind::Cast;
            }
            ControlFlow::Continue(())
        }

        fn pre_visit_ident(&mut self, ident: &mut Ident) -> ControlFlow<()> {
            ident.value = ident_key(ident);
            ident.quote_style = None;
            ControlFlow::Continue(())
        }
    }

    let mut key = expr.clone();
    let _ = key.visit(&mut Identifiers);
    key
}
