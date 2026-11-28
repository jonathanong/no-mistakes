use super::conditions::boolean_result_is_fixed;
use super::function_clauses::{
    coalesce_arguments, filter_skips_inputs, is_eager, split_coalesce_arguments,
    visit_eager_functions, visit_function_inputs, visit_ordered_set_arguments,
};
use super::windows::{window_expressions, WindowPlan};
use super::{query::walk_exists, walk_query, LookupHistory, ScopeStack};
use crate::idents::function_arg_expr;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, Function, Query, Visit, Visitor, WindowType};
mod case;
use std::ops::ControlFlow;

pub(super) fn walk_nested_queries(
    node: &impl Visit,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    walk_nested_with_context(node, scopes, out, LookupHistory::new(), false, None);
}

pub(super) fn walk_nested_with_context(
    node: &impl Visit,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
    guards: LookupHistory,
    empty_input: bool,
    windows: Option<&WindowPlan>,
) {
    if guards.is_reachable() {
        let _ = node.visit(&mut Nested {
            query_depth: 0,
            blocked_expression_depth: 0,
            scopes,
            out,
            guards,
            empty_input,
            windows,
        });
    }
}

struct Nested<'a> {
    query_depth: usize,
    blocked_expression_depth: usize,
    scopes: &'a mut ScopeStack,
    out: &'a mut Vec<SqlDuplicateKeyedLookupFact>,
    guards: LookupHistory,
    empty_input: bool,
    windows: Option<&'a WindowPlan>,
}

impl Nested<'_> {
    fn walk_conditional_children(&mut self, expr: &Expr) {
        match expr {
            Expr::BinaryOp { left, op, right } if boolean_result_is_fixed(left, op, right) => {
                self.blocked_expression_depth = 1;
            }
            Expr::Exists { subquery, .. } => {
                self.blocked_expression_depth = 1;
                walk_exists(subquery, self.scopes, self.out);
            }
            Expr::Function(function)
                if self.empty_input && function.over.is_none() && is_eager(function) =>
            {
                self.blocked_expression_depth = 1;
                visit_ordered_set_arguments(function, &mut |arg| self.walk_child(arg));
            }
            Expr::Function(function)
                if function.over.is_some() || filter_skips_inputs(function) =>
            {
                self.blocked_expression_depth = 1;
                self.walk_function(expr, function);
            }
            Expr::Function(function) => {
                if let Some(arguments) = coalesce_arguments(function) {
                    let (evaluated, skipped) = split_coalesce_arguments(arguments);
                    if !skipped.is_empty() {
                        self.blocked_expression_depth = 1;
                        for argument in evaluated.iter().filter_map(function_arg_expr) {
                            self.walk_child(argument);
                        }
                    }
                }
            }
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                self.blocked_expression_depth = 1;
                self.walk_case(operand.as_deref(), conditions, else_result.as_deref());
            }
            _ => {}
        }
    }

    fn walk_child(&mut self, node: &impl Visit) {
        walk_nested_with_context(
            node,
            self.scopes,
            self.out,
            self.guards.guard_only(),
            self.empty_input,
            self.windows,
        );
    }

    fn walk_function(&mut self, expr: &Expr, function: &Function) {
        if self.windows.is_some_and(|windows| windows.skips_call(expr)) {
            return;
        }
        self.walk_child(&function.filter);
        visit_ordered_set_arguments(function, &mut |arg| self.walk_child(arg));
        visit_function_inputs(function, &mut |arg| self.walk_child(arg));
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

    fn walk_eager_children(&mut self, skipped: &Expr) {
        visit_eager_functions(skipped, &mut |eager| {
            if !matches!(eager, Expr::Function(function) if function.over.is_some()) {
                self.walk_child(eager);
            }
        });
    }
}

impl Visitor for Nested<'_> {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if self.query_depth == 0 && self.blocked_expression_depth == 0 {
            walk_query(query, self.scopes, self.out);
        }
        self.query_depth += 1;
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.query_depth -= 1;
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if self.blocked_expression_depth > 0 {
            self.blocked_expression_depth += 1;
        } else if self.query_depth == 0 {
            self.walk_conditional_children(expr);
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<()> {
        self.blocked_expression_depth = self.blocked_expression_depth.saturating_sub(1);
        ControlFlow::Continue(())
    }
}
