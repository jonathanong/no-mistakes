mod identity;

use super::grouping;
use super::join::register_table_with_joins;
use super::scope::ScopeStack;
use super::subquery::inspect_expr;
use super::windows::referenced_expressions;
use super::{select_item_expr, walk_nested_queries, LookupHistory};
use crate::fx::FxHashSet;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use identity::normalize_target;
pub(super) use identity::target_identity;
use sqlparser::ast::{Distinct, Expr, Select, WindowSpec};
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};

#[derive(Default)]
pub(super) struct SelectTargets<'a> {
    projection: Vec<Cow<'a, Expr>>,
    keys: OnceCell<RefCell<FxHashSet<Expr>>>,
    windows: RefCell<FxHashSet<WindowSpec>>,
}

impl<'a> SelectTargets<'a> {
    fn new(projection: Vec<Cow<'a, Expr>>) -> Self {
        Self {
            projection,
            ..Self::default()
        }
    }

    pub(super) fn reuses_window(&self, spec: &WindowSpec) -> bool {
        if spec.window_frame.is_none() {
            return false;
        }
        let mut key = spec.clone();
        normalize_target(&mut key);
        !self.windows.borrow_mut().insert(key)
    }

    pub(super) fn reuses(&self, expr: &Expr) -> bool {
        !self
            .keys
            .get_or_init(|| {
                RefCell::new(
                    self.projection
                        .iter()
                        .map(|expr| target_identity(expr))
                        .collect(),
                )
            })
            .borrow_mut()
            .insert(target_identity(expr))
    }
}

pub(super) fn walk_select(
    select: &Select,
    order_by: &[&Expr],
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> LookupHistory {
    scopes.push_frame();
    let mut seen = LookupHistory::new();
    for table in &select.from {
        register_table_with_joins(table, scopes, &mut seen, out);
    }
    let outer = scopes.snapshot();
    let group_by = grouping::targets(&select.group_by);
    let reuse = grouping::GroupReuse::new(&group_by);
    let projection = select
        .projection
        .iter()
        .filter_map(select_item_expr)
        .map(|expr| reuse.filter(expr))
        .collect();
    let targets = SelectTargets::new(projection);
    let having = select.having.as_ref().map(|expr| reuse.filter(expr));
    for expr in inspectable(select, order_by, &targets, &group_by, having.as_deref()) {
        inspect_expr(expr, &outer, &targets, &mut seen, out);
        walk_nested_queries(expr, scopes, out);
    }
    scopes.pop_frame();
    seen
}

fn inspectable<'a>(
    select: &'a Select,
    order_by: &'a [&'a Expr],
    select_targets: &'a SelectTargets<'a>,
    grouping_targets: &'a [&'a Expr],
    having: Option<&'a Expr>,
) -> impl Iterator<Item = &'a Expr> {
    let distinct_on = match &select.distinct {
        Some(Distinct::On(expressions)) => expressions.as_slice(),
        _ => &[],
    };
    let projection = select_targets.projection.iter().map(Cow::as_ref);

    let clause_target = |expr| (expr, true);
    let independent = |expr| (expr, false);
    distinct_on
        .iter()
        .map(clause_target)
        .chain(projection.map(independent))
        .chain(select.selection.iter().map(independent))
        .chain(grouping_targets.iter().copied().map(clause_target))
        .chain(having.into_iter().map(independent))
        .chain(referenced_expressions(select, distinct_on, order_by))
        .chain(order_by.iter().copied().map(clause_target))
        .filter_map(move |(expr, can_reuse)| {
            (!can_reuse || !select_targets.reuses(expr)).then_some(expr)
        })
}
