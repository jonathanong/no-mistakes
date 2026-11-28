use super::function_clauses::{
    coalesce_arguments, condition_is_never_true, split_coalesce_arguments, visit_eager_functions,
};
use super::{walk_query, ScopeStack};
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
                operand: None,
                conditions,
                else_result,
                ..
            } if conditions
                .iter()
                .any(|branch| condition_is_never_true(&branch.condition)) =>
            {
                self.blocked_expression_depth = 1;
                for branch in conditions {
                    if condition_is_never_true(&branch.condition) {
                        self.walk_eager_children(&branch.result);
                    } else {
                        walk_nested_queries(&branch.condition, self.scopes, self.out);
                        walk_nested_queries(&branch.result, self.scopes, self.out);
                    }
                }
                walk_nested_queries(else_result, self.scopes, self.out);
            }
            _ => {}
        }
    }

    fn walk_eager_children(&mut self, skipped: &Expr) {
        visit_eager_functions(skipped, &mut |eager| {
            walk_nested_queries(eager, self.scopes, self.out);
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
