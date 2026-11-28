use super::conditions::{boolean_result_is_fixed, case_condition, visit_case_children};
use super::function_clauses::{
    coalesce_arguments, filter_skips_inputs, has_optional_window_default, null_position_inputs,
    split_coalesce_arguments, visit_eager_functions, visit_function_inputs,
    visit_ordered_set_arguments,
};
use super::{query::walk_exists, walk_query, ScopeStack};
use crate::idents::function_arg_expr;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, Query, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn walk_nested_queries(
    node: &impl Visit,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let _ = node.visit(&mut Nested {
        query_depth: 0,
        blocked_expression_depth: 0,
        scopes,
        out,
    });
}

struct Nested<'a> {
    query_depth: usize,
    blocked_expression_depth: usize,
    scopes: &'a mut ScopeStack,
    out: &'a mut Vec<SqlDuplicateKeyedLookupFact>,
}

impl Nested<'_> {
    fn walk_conditional_children(&mut self, expr: &Expr) {
        match expr {
            Expr::BinaryOp { left, op, right } if boolean_result_is_fixed(left, op, right) => {
                self.blocked_expression_depth = 1;
                self.walk_eager_children(expr);
            }
            Expr::Exists { subquery, .. } => {
                self.blocked_expression_depth = 1;
                walk_exists(subquery, self.scopes, self.out);
            }
            Expr::Function(function)
                if filter_skips_inputs(function)
                    || has_optional_window_default(function)
                    || null_position_inputs(function).is_some() =>
            {
                self.blocked_expression_depth = 1;
                walk_nested_queries(&function.filter, self.scopes, self.out);
                visit_ordered_set_arguments(function, &mut |arg| {
                    walk_nested_queries(arg, self.scopes, self.out)
                });
                visit_function_inputs(function, &mut |arg| {
                    walk_nested_queries(arg, self.scopes, self.out)
                });
                walk_nested_queries(&function.over, self.scopes, self.out);
            }
            Expr::Function(function) => {
                if let Some(arguments) = coalesce_arguments(function) {
                    let (evaluated, skipped) = split_coalesce_arguments(arguments);
                    if !skipped.is_empty() {
                        self.blocked_expression_depth = 1;
                        for argument in evaluated.iter().filter_map(function_arg_expr) {
                            walk_nested_queries(argument, self.scopes, self.out);
                        }
                        for argument in skipped.iter().filter_map(function_arg_expr) {
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
            } if conditions
                .iter()
                .any(|branch| case_condition(operand.as_deref(), &branch.condition).is_some()) =>
            {
                self.blocked_expression_depth = 1;
                if let Some(operand) = operand {
                    walk_nested_queries(operand, self.scopes, self.out);
                }
                visit_case_children(
                    operand.as_deref(),
                    conditions,
                    else_result.as_deref(),
                    |expr, reachable| {
                        if reachable {
                            walk_nested_queries(expr, self.scopes, self.out);
                        }
                    },
                );
            }
            _ => {}
        }
    }

    fn walk_eager_children(&mut self, skipped: &Expr) {
        visit_eager_functions(skipped, &mut |eager| {
            if !matches!(eager, Expr::Function(function) if function.over.is_some()) {
                walk_nested_queries(eager, self.scopes, self.out);
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
