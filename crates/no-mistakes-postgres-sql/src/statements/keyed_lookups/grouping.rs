mod targets;

use super::function_clauses::is_eager;
use super::select_exprs::target_identity;
use crate::fx::{FxHashMap, FxHashSet};
use crate::idents::unwrap_expr;
use sqlparser::ast::{Expr, GroupByExpr, Query, Value, VisitMut, VisitorMut};
use std::borrow::Cow;
use std::mem::{discriminant, Discriminant};
use std::ops::ControlFlow;
pub(super) use targets::GroupingTargets;

pub(super) fn can_omit_keys(group_by: &GroupByExpr) -> bool {
    matches!(group_by, GroupByExpr::Expressions(expressions, _) if expressions.iter().any(|expr|
        matches!(unwrap_expr(expr), Expr::GroupingSets(_) | Expr::Rollup(_) | Expr::Cube(_))
    ))
}

const GROUPED_VALUE_PREFIX: &str = "GROUP BY ";

pub(super) fn grouped_position(expr: &Expr) -> Option<usize> {
    let Value::Placeholder(name) = super::conditions::literal(expr)? else {
        return None;
    };
    name.strip_prefix(GROUPED_VALUE_PREFIX)?.parse().ok()
}

pub(super) struct GroupReuse {
    keys: FxHashMap<Expr, usize>,
    kinds: FxHashSet<Discriminant<Expr>>,
}

impl GroupReuse {
    pub(super) fn new(grouping: &GroupingTargets<'_>) -> Self {
        let keys: FxHashMap<_, _> = grouping
            .all
            .iter()
            .enumerate()
            .filter_map(|(position, expr)| {
                let key = target_identity(expr);
                if matches!(key, Expr::Value(_)) && grouping.required.contains(&key) {
                    return None;
                }
                Some((key, position))
            })
            .collect();
        let kinds = keys.keys().map(discriminant).collect();
        Self { keys, kinds }
    }

    pub(super) fn filter<'a>(&self, expr: &'a Expr) -> Cow<'a, Expr> {
        if self.keys.is_empty() {
            return Cow::Borrowed(expr);
        }
        let mut filtered = expr.clone();
        let _ = filtered.visit(&mut GroupedExpressions {
            reuse: self,
            depth: 0,
        });
        Cow::Owned(filtered)
    }
}

struct GroupedExpressions<'a> {
    reuse: &'a GroupReuse,
    depth: usize,
}

impl VisitorMut for GroupedExpressions<'_> {
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
        if self.depth == 0 && self.reuse.kinds.contains(&discriminant(unwrap_expr(expr))) {
            if let Some(position) = self.reuse.keys.get(&target_identity(expr)) {
                *expr = Expr::Value(
                    Value::Placeholder(format!("{GROUPED_VALUE_PREFIX}{position}"))
                        .with_empty_span(),
                );
            }
        }
        if let Expr::Function(function) = expr {
            if function.over.is_some() {
                if self.depth == 0 {
                    let _ = function.args.visit(self);
                    let _ = function.filter.visit(self);
                }
                self.depth += 1;
            } else if is_eager(function) {
                if self.depth == 0 && !function.within_group.is_empty() {
                    let _ = function.args.visit(self);
                }
                self.depth += 1;
            }
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
        if matches!(expr, Expr::Function(function) if function.over.is_some() || is_eager(function))
        {
            self.depth -= 1;
        }
        ControlFlow::Continue(())
    }
}
