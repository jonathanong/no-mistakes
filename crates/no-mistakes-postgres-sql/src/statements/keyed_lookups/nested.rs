use super::conditions::boolean_result_is_fixed;
use super::function_clauses::{
    coalesce_arguments, is_eager, split_coalesce_arguments, strict_null_arguments,
    visit_ordered_set_arguments,
};
use super::windows::WindowPlan;
use super::{query::walk_exists, walk_query, LookupHistory, ScopeStack};
use crate::idents::function_arg_expr;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, Query, Visit, Visitor};
mod case;
mod functions;
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
    Nested {
        query_depth: 0,
        blocked_expression_depth: 0,
        scopes,
        out,
        guards,
        empty_input,
        windows,
        earlier_aggregates_visited: false,
    }
    .walk(node);
}

struct Nested<'a> {
    query_depth: usize,
    blocked_expression_depth: usize,
    scopes: &'a mut ScopeStack,
    out: &'a mut Vec<SqlDuplicateKeyedLookupFact>,
    guards: LookupHistory,
    empty_input: bool,
    windows: Option<&'a WindowPlan>,
    earlier_aggregates_visited: bool,
}

impl Nested<'_> {
    fn walk(&mut self, node: &impl Visit) {
        if self.guards.is_reachable() {
            let _ = node.visit(self);
        }
    }

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
                if (self.empty_input || self.earlier_aggregates_visited)
                    && function.over.is_none()
                    && is_eager(function) =>
            {
                self.blocked_expression_depth = 1;
                if !self.earlier_aggregates_visited {
                    visit_ordered_set_arguments(function, &mut |arg| self.walk_child(arg));
                }
            }
            Expr::Function(function) if function.over.is_some() || function.filter.is_some() => {
                self.blocked_expression_depth = 1;
                self.walk_function(expr, function);
            }
            Expr::Function(function) if strict_null_arguments(function).is_some() => {
                self.blocked_expression_depth = 1;
            }
            Expr::Function(function) => {
                if let Some(arguments) = coalesce_arguments(function) {
                    self.blocked_expression_depth = 1;
                    let (evaluated, _) = split_coalesce_arguments(arguments);
                    let outer = self.scopes.snapshot();
                    let mut guards = self.guards.guard_only();
                    for argument in evaluated.iter().filter_map(function_arg_expr) {
                        if guards.is_reachable() {
                            self.walk_child_with_guards(argument, guards.guard_only());
                            guards.restrict_to_null(argument, &outer);
                        } else {
                            self.walk_eager_children(argument);
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
        self.walk_child_with_guards(node, self.guards.guard_only());
    }

    fn walk_child_with_guards(&mut self, node: &impl Visit, guards: LookupHistory) {
        Nested {
            query_depth: 0,
            blocked_expression_depth: 0,
            scopes: self.scopes,
            out: self.out,
            guards,
            empty_input: self.empty_input,
            windows: self.windows,
            earlier_aggregates_visited: self.earlier_aggregates_visited,
        }
        .walk(node);
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
