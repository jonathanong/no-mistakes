use super::function_clauses::is_eager;
use super::select_exprs::target_identity;
use crate::fx::FxHashSet;
use crate::idents::unwrap_expr;
use sqlparser::ast::{Expr, GroupByExpr, Query, Value, VisitMut, VisitorMut};
use std::borrow::Cow;
use std::mem::{discriminant, Discriminant};
use std::ops::ControlFlow;

pub(super) fn targets(group_by: &GroupByExpr) -> Vec<&Expr> {
    let mut targets = Vec::new();
    if let GroupByExpr::Expressions(expressions, _) = group_by {
        for expr in expressions {
            collect_targets(expr, &mut targets);
        }
    }
    targets
}

fn collect_targets<'a>(expr: &'a Expr, targets: &mut Vec<&'a Expr>) {
    match unwrap_expr(expr) {
        Expr::GroupingSets(sets) | Expr::Cube(sets) | Expr::Rollup(sets) => {
            for member in sets.iter().flatten() {
                collect_targets(member, targets);
            }
        }
        Expr::Tuple(members) => {
            for member in members {
                collect_targets(member, targets);
            }
        }
        _ => targets.push(expr),
    }
}

pub(super) struct GroupReuse {
    keys: FxHashSet<Expr>,
    kinds: FxHashSet<Discriminant<Expr>>,
}

impl GroupReuse {
    pub(super) fn new(grouping: &[&Expr]) -> Self {
        let keys: FxHashSet<_> = grouping.iter().map(|expr| target_identity(expr)).collect();
        let kinds = keys.iter().map(discriminant).collect();
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
        if self.depth == 0
            && self.reuse.kinds.contains(&discriminant(unwrap_expr(expr)))
            && self.reuse.keys.contains(&target_identity(expr))
        {
            *expr = Expr::Value(Value::Null.with_empty_span());
        }
        if matches!(expr, Expr::Function(function) if function.over.is_none() && is_eager(function))
        {
            self.depth += 1;
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
        if matches!(expr, Expr::Function(function) if function.over.is_none() && is_eager(function))
        {
            self.depth -= 1;
        }
        ControlFlow::Continue(())
    }
}
