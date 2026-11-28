use super::super::conditions::collect_pruned_children;
use super::is_eager;
use crate::fx::FxHashSet;
use sqlparser::ast::Expr;

pub(in crate::statements::keyed_lookups) fn visit_eager_functions(
    expr: &Expr,
    visit: &mut impl FnMut(&Expr),
) {
    visit_early_functions(expr, true, visit);
}

pub(in crate::statements::keyed_lookups) fn visit_ordinary_aggregates(
    expr: &Expr,
    visit: &mut impl FnMut(&Expr),
) {
    visit_early_functions(expr, false, visit);
}

fn visit_early_functions(expr: &Expr, include_windows: bool, visit: &mut impl FnMut(&Expr)) {
    use sqlparser::ast::{Query, Visit, Visitor};
    use std::ops::ControlFlow;

    struct EagerFunctions<'a, F> {
        visit: &'a mut F,
        include_windows: bool,
        depth: usize,
        blocked_depth: usize,
        blocked_roots: FxHashSet<*const Expr>,
    }

    impl<F> EagerFunctions<'_, F> {
        fn runs_early(&self, expr: &Expr) -> bool {
            matches!(expr, Expr::Function(function)
                if is_eager(function) && (self.include_windows || function.over.is_none()))
        }
    }

    impl<F: FnMut(&Expr)> Visitor for EagerFunctions<'_, F> {
        type Break = ();

        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.depth += 1;
            ControlFlow::Continue(())
        }

        fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.depth -= 1;
            ControlFlow::Continue(())
        }

        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if self.blocked_depth > 0 || self.blocked_roots.remove(&std::ptr::from_ref(expr)) {
                self.blocked_depth += 1;
                return ControlFlow::Continue(());
            }
            if self.depth == 0 {
                collect_pruned_children(expr, &mut self.blocked_roots);
            }
            if self.runs_early(expr) {
                if self.depth == 0 {
                    (self.visit)(expr);
                }
                self.depth += 1;
            }
            ControlFlow::Continue(())
        }

        fn post_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if self.blocked_depth > 0 {
                self.blocked_depth -= 1;
                return ControlFlow::Continue(());
            }
            if self.runs_early(expr) {
                self.depth -= 1;
            }
            ControlFlow::Continue(())
        }
    }

    let _ = expr.visit(&mut EagerFunctions {
        visit,
        include_windows,
        depth: 0,
        blocked_depth: 0,
        blocked_roots: FxHashSet::default(),
    });
}
