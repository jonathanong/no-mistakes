use super::super::function_clauses::{
    empty_aggregate_frame, filter_skips_inputs, visit_eager_functions, visit_function_filter,
    visit_function_inputs, visit_ordered_set_arguments,
};
use super::super::windows::window_expressions;
use super::Nested;
use sqlparser::ast::{Expr, Function, WindowType};

impl Nested<'_> {
    pub(super) fn walk_function(&mut self, expr: &Expr, function: &Function) {
        if self.windows.is_some_and(|windows| windows.skips_call(expr)) {
            return;
        }
        visit_function_filter(function, &mut |filter| self.walk_child(filter));
        visit_ordered_set_arguments(function, &mut |arg| self.walk_child(arg));
        self.walk_function_inputs(function);
        if let Some(WindowType::WindowSpec(spec)) = &function.over {
            if let Some(needs_frame) = self
                .windows
                .map_or(Some(true), |windows| windows.take_inline_frame(spec, true))
            {
                for (expr, _) in window_expressions(spec, needs_frame) {
                    self.walk_child(expr);
                }
            }
        }
    }

    fn walk_function_inputs(&mut self, function: &Function) {
        let Some(filter) = function
            .filter
            .as_ref()
            .filter(|_| !filter_skips_inputs(function) && !empty_aggregate_frame(function))
        else {
            visit_function_inputs(function, &mut |arg| self.walk_child(arg));
            return;
        };
        let earlier_aggregates_visited = self.earlier_aggregates_visited;
        if function.over.is_some() && !earlier_aggregates_visited {
            visit_function_inputs(function, &mut |arg| self.walk_eager_children(arg));
            self.earlier_aggregates_visited = true;
        }
        let previous = self.guards.guard_only();
        self.guards.restrict(filter, &self.scopes.snapshot(), true);
        visit_function_inputs(function, &mut |arg| self.walk_child(arg));
        self.guards = previous;
        self.earlier_aggregates_visited = earlier_aggregates_visited;
    }

    pub(super) fn walk_eager_children(&mut self, skipped: &Expr) {
        visit_eager_functions(skipped, &mut |eager| {
            if !matches!(eager, Expr::Function(function) if function.over.is_some()) {
                self.walk_child(eager);
            }
        });
    }
}
