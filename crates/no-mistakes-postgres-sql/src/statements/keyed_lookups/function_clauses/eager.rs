use super::super::conditions::{boolean_result_is_fixed, visit_case_children};
use super::{coalesce_arguments, is_eager, split_coalesce_arguments};
use crate::fx::FxHashSet;
use crate::idents::function_arg_expr;
use sqlparser::ast::Expr;

pub(in crate::statements::keyed_lookups) fn visit_eager_functions(
    expr: &Expr,
    visit: &mut impl FnMut(&Expr),
) {
    use sqlparser::ast::{Query, Visit, Visitor};
    use std::ops::ControlFlow;

    struct EagerFunctions<'a, F> {
        visit: &'a mut F,
        depth: usize,
        blocked_depth: usize,
        blocked_roots: FxHashSet<*const Expr>,
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
                match expr {
                    Expr::BinaryOp { left, op, right }
                        if boolean_result_is_fixed(left, op, right) =>
                    {
                        self.blocked_roots
                            .extend([left.as_ref(), right.as_ref()].map(std::ptr::from_ref));
                    }
                    Expr::Case {
                        operand,
                        conditions,
                        else_result,
                        ..
                    } => {
                        visit_case_children(
                            operand.as_deref(),
                            conditions,
                            else_result.as_deref(),
                            |child, reachable| {
                                if !reachable {
                                    self.blocked_roots.insert(std::ptr::from_ref(child));
                                }
                            },
                        );
                    }
                    Expr::Function(function) => {
                        if let Some(arguments) = coalesce_arguments(function) {
                            let (_, skipped) = split_coalesce_arguments(arguments);
                            self.blocked_roots.extend(
                                skipped
                                    .iter()
                                    .filter_map(function_arg_expr)
                                    .map(std::ptr::from_ref),
                            );
                        }
                    }
                    _ => {}
                }
            }
            if matches!(expr, Expr::Function(function) if is_eager(function)) {
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
            if matches!(expr, Expr::Function(function) if is_eager(function)) {
                self.depth -= 1;
            }
            ControlFlow::Continue(())
        }
    }

    let _ = expr.visit(&mut EagerFunctions {
        visit,
        depth: 0,
        blocked_depth: 0,
        blocked_roots: FxHashSet::default(),
    });
}
