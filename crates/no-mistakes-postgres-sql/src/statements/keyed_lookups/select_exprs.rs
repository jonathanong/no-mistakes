use super::select_item_expr;
use super::table::lookup_table;
use super::windows::referenced_expressions;
use crate::fx::FxHashSet;
use crate::idents::{ident_key, object_name_identity, relation_suffix_name, unwrap_expr};
use sqlparser::ast::{
    CastKind, Distinct, Expr, GroupByExpr, Ident, Query, Select, SelectItem, SetExpr, VisitMut,
    VisitorMut,
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
    let mut grouping_targets = Vec::new();
    if let GroupByExpr::Expressions(expressions, _) = &select.group_by {
        for expr in expressions {
            group_targets(expr, &mut grouping_targets);
        }
    }

    let projection = select.projection.iter().filter_map(select_item_expr);
    let mut targets = FxHashSet::default();

    let clause_target = |expr| (expr, true);
    let independent = |expr| (expr, false);
    distinct_on
        .iter()
        .map(clause_target)
        .chain(projection.map(independent))
        .chain(select.selection.iter().map(independent))
        .chain(grouping_targets.into_iter().map(clause_target))
        .chain(select.having.iter().map(independent))
        .chain(referenced_expressions(select, distinct_on, order_by))
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

fn group_targets<'a>(expr: &'a Expr, targets: &mut Vec<&'a Expr>) {
    match unwrap_expr(expr) {
        Expr::GroupingSets(sets) | Expr::Cube(sets) | Expr::Rollup(sets) => {
            for member in sets.iter().flatten() {
                group_targets(member, targets);
            }
        }
        _ => targets.push(expr),
    }
}

fn target_identity(expr: &Expr) -> Expr {
    struct Identifiers;
    impl VisitorMut for Identifiers {
        type Break = ();

        fn pre_visit_query(&mut self, query: &mut Query) -> ControlFlow<()> {
            if let SetExpr::Select(select) = query.body.as_mut() {
                normalize_local_columns(select);
            }
            ControlFlow::Continue(())
        }

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

fn normalize_local_columns(select: &mut Select) {
    let [table] = select.from.as_slice() else {
        return;
    };
    if !table.joins.is_empty() {
        return;
    }
    let Some((name, alias)) = lookup_table(&table.relation) else {
        return;
    };
    let qualifier = alias.map_or_else(
        || relation_suffix_name(&object_name_identity(name)),
        |alias| ident_key(&alias.name),
    );

    struct LocalColumns<'a> {
        qualifier: &'a str,
        depth: usize,
    }

    impl VisitorMut for LocalColumns<'_> {
        type Break = ();

        fn pre_visit_query(&mut self, _: &mut Query) -> ControlFlow<()> {
            self.depth += 1;
            ControlFlow::Continue(())
        }

        fn post_visit_query(&mut self, _: &mut Query) -> ControlFlow<()> {
            self.depth -= 1;
            ControlFlow::Continue(())
        }

        fn pre_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
            if self.depth == 0 {
                if let Expr::CompoundIdentifier(parts) = expr {
                    if let [qualifier, column] = parts.as_slice() {
                        if ident_key(qualifier) == self.qualifier {
                            *expr = Expr::Identifier(column.clone());
                        }
                    }
                }
            }
            ControlFlow::Continue(())
        }
    }

    let mut visitor = LocalColumns {
        qualifier: &qualifier,
        depth: 0,
    };
    for expr in select
        .projection
        .iter_mut()
        .filter_map(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
            _ => None,
        })
        .chain(select.selection.iter_mut())
    {
        let _ = expr.visit(&mut visitor);
    }
}
