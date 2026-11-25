use super::actions::{find_action_in_expr, find_action_in_stmts, Action, ActionScope};
use super::expr::{call_member, check_state_expr, match_listing, queue_receiver, unwrap_expr};
use super::extract::{record, FnEnv};
use super::item::ListingItem;
use super::{JobStatePattern, JobStateSite};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::object_bindings::collect_names;
use oxc_ast::ast::{CallExpression, ConditionalExpression, Expression, FormalParameters};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashSet;

pub(super) fn check_expr_for_actions(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    env: &FnEnv,
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    let mut checker = ExprChecker {
        visitor,
        env,
        sites,
        seen,
    };
    checker.visit_expression(expr);
}

struct ExprChecker<'c, 'b> {
    visitor: &'b QueueVisitor<'c>,
    env: &'b FnEnv,
    sites: &'b mut Vec<JobStateSite>,
    seen: &'b mut HashSet<(usize, JobStatePattern)>,
}

impl<'a> Visit<'a> for ExprChecker<'_, '_> {
    fn visit_conditional_expression(&mut self, cond: &ConditionalExpression<'a>) {
        if let Some(binding) = check_state_expr(self.visitor, &cond.test, self.env) {
            let names: Vec<_> = self.env.job_vars.keys().map(String::as_str).collect();
            let scope = ActionScope::StateGuard(&names);
            let action = find_action_in_expr(self.visitor, &cond.consequent, scope)
                .or_else(|| find_action_in_expr(self.visitor, &cond.alternate, scope));
            if let Some(action_binding) = action {
                record(
                    self.visitor,
                    cond.span.start,
                    binding,
                    action_binding,
                    JobStatePattern::GetJobStateGuard,
                    self.sites,
                    self.seen,
                );
            }
        }
        walk::walk_conditional_expression(self, cond);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some((object, method)) = call_member(call) {
            if matches!(method, "map" | "forEach" | "flatMap") {
                if let Some(binding) = listing_binding(self.visitor, object, self.env) {
                    if let Some(action_binding) = callback_listing_action(self.visitor, call) {
                        record(
                            self.visitor,
                            call.span.start,
                            binding,
                            action_binding,
                            JobStatePattern::ListingFeedsAction,
                            self.sites,
                            self.seen,
                        );
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                if let Some(action_q) = queue_receiver(self.visitor, object, call.span.start) {
                    if let Some(listing_q) = call.arguments.iter().find_map(|arg| {
                        arg.as_expression()
                            .and_then(|e| expr_uses_listing(self.visitor, e, self.env))
                    }) {
                        record(
                            self.visitor,
                            call.span.start,
                            listing_q,
                            Action::Enqueue(action_q),
                            JobStatePattern::ListingFeedsAction,
                            self.sites,
                            self.seen,
                        );
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

pub(super) fn listing_binding(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    env: &FnEnv,
) -> Option<ProducerBinding> {
    if let Some(binding) = match_listing(visitor, expr) {
        return Some(binding);
    }
    if let Expression::Identifier(id) = unwrap_expr(expr) {
        return env.listing_vars.get(id.name.as_str()).cloned();
    }
    None
}

fn expr_uses_listing(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    env: &FnEnv,
) -> Option<ProducerBinding> {
    struct Finder<'c, 'b> {
        visitor: &'b QueueVisitor<'c>,
        env: &'b FnEnv,
        found: Option<ProducerBinding>,
    }
    impl<'a> Visit<'a> for Finder<'_, '_> {
        fn visit_expression(&mut self, expr: &Expression<'a>) {
            if self.found.is_some() {
                return;
            }
            if let Some(binding) = listing_binding(self.visitor, expr, self.env) {
                self.found = Some(binding);
                return;
            }
            walk::walk_expression(self, expr);
        }
    }
    let mut finder = Finder {
        visitor,
        env,
        found: None,
    };
    finder.visit_expression(expr);
    finder.found
}

fn callback_listing_action(
    visitor: &QueueVisitor<'_>,
    call: &CallExpression<'_>,
) -> Option<Action> {
    let cb = call.arguments.first()?.as_expression()?;
    match unwrap_ts_wrappers(cb) {
        Expression::ArrowFunctionExpression(arrow) => {
            let names = item_names(&arrow.params);
            let scope = ActionScope::Listing(ListingItem {
                names: &names,
                start: arrow.span.start,
            });
            if let Some(expr) = arrow.body.as_expression() {
                return find_action_in_expr(visitor, expr, scope);
            }
            find_action_in_stmts(
                visitor,
                crate::ast::arrow_function_body_statements(&arrow.body)?,
                scope,
            )
        }
        Expression::FunctionExpression(func) => {
            let names = item_names(&func.params);
            let scope = ActionScope::Listing(ListingItem {
                names: &names,
                start: func.span.start,
            });
            find_action_in_stmts(visitor, &func.body.as_ref()?.statements, scope)
        }
        _ => None,
    }
}

fn item_names<'a>(params: &FormalParameters<'a>) -> Vec<&'a str> {
    let mut names = Vec::new();
    if let Some(item) = params.items.first() {
        collect_names(&item.pattern, &mut names);
    }
    names
}
