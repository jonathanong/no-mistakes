use super::function_clauses::{
    coalesce_arguments, split_coalesce_arguments, visit_eager_functions,
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
            if let Expr::Function(function) = expr {
                if let Some(arguments) = coalesce_arguments(function) {
                    let (evaluated, skipped) = split_coalesce_arguments(arguments);
                    if !skipped.is_empty() {
                        self.blocked_expression_depth = 1;
                        for argument in evaluated.iter().filter_map(function_arg_expr) {
                            walk_nested_queries(argument, self.scopes, self.out);
                        }
                        for argument in skipped.iter().filter_map(function_arg_expr) {
                            visit_eager_functions(argument, &mut |eager| {
                                walk_nested_queries(eager, self.scopes, self.out);
                            });
                        }
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<()> {
        self.blocked_expression_depth = self.blocked_expression_depth.saturating_sub(1);
        ControlFlow::Continue(())
    }
}
