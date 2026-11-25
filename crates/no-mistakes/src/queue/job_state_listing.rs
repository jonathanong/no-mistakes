use super::actions::{find_in_expr, find_in_stmts, ActionMode};
use super::expr::{call_member, check_state_expr, derived_listing_binding, pattern_names};
use super::extract::{ExtractCtx, FnEnv};
use super::JobStatePattern;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::queue::extract_model::ProducerBinding;
use oxc_ast::ast::{CallExpression, ConditionalExpression, Expression};
use oxc_ast_visit::{walk, Visit};

pub(super) fn check_expr_for_actions(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    env: &FnEnv,
) {
    ExprChecker { ctx, env }.visit_expression(expr);
}

struct ExprChecker<'a, 'b, 'c> {
    ctx: &'a mut ExtractCtx<'b, 'c>,
    env: &'a FnEnv,
}

impl<'a> Visit<'a> for ExprChecker<'_, '_, '_> {
    fn visit_conditional_expression(&mut self, cond: &ConditionalExpression<'a>) {
        if let Some(matched) = check_state_expr(self.ctx, &cond.test, self.env) {
            let names = self.env.job_names_for(matched.job_id);
            let mode = ActionMode::Guarded;
            let action = find_in_expr(self.ctx, &cond.consequent, &names, mode)
                .or_else(|| find_in_expr(self.ctx, &cond.alternate, &names, mode));
            if let Some(action_binding) = action {
                self.ctx.record(
                    cond.span.start,
                    matched.binding,
                    action_binding,
                    JobStatePattern::GetJobStateGuard,
                );
            }
        }
        walk::walk_conditional_expression(self, cond);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some((object, method)) = call_member(call) {
            if matches!(method, "map" | "forEach" | "flatMap") {
                if let Some(binding) = derived_listing_binding(self.ctx, object, self.env) {
                    if let Some(action_binding) = callback_listing_action(self.ctx, call) {
                        self.ctx.record(
                            call.span.start,
                            binding,
                            action_binding,
                            JobStatePattern::ListingFeedsAction,
                        );
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                if let Some(action_q) = self.ctx.queue_receiver(object, call.span.start) {
                    if let Some(listing_q) = call.arguments.iter().find_map(|arg| {
                        arg.as_expression()
                            .and_then(|e| expr_uses_listing(self.ctx, e, self.env))
                    }) {
                        self.ctx.record(
                            call.span.start,
                            listing_q,
                            Some(action_q),
                            JobStatePattern::ListingFeedsAction,
                        );
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

fn expr_uses_listing(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    env: &FnEnv,
) -> Option<ProducerBinding> {
    struct Finder<'a, 'b, 'c> {
        ctx: &'a mut ExtractCtx<'b, 'c>,
        env: &'a FnEnv,
        found: Option<ProducerBinding>,
    }
    impl<'a> Visit<'a> for Finder<'_, '_, '_> {
        fn visit_expression(&mut self, expr: &Expression<'a>) {
            if self.found.is_some() {
                return;
            }
            if let Some(binding) = derived_listing_binding(self.ctx, expr, self.env) {
                self.found = Some(binding);
                return;
            }
            walk::walk_expression(self, expr);
        }
    }
    let mut finder = Finder {
        ctx,
        env,
        found: None,
    };
    finder.visit_expression(expr);
    finder.found
}

fn callback_listing_action(
    ctx: &mut ExtractCtx<'_, '_>,
    call: &CallExpression<'_>,
) -> Option<Option<ProducerBinding>> {
    let cb = call.arguments.first()?.as_expression()?;
    match unwrap_ts_wrappers(cb) {
        Expression::ArrowFunctionExpression(arrow) => {
            let params: Vec<_> = arrow
                .params
                .items
                .iter()
                .flat_map(|p| pattern_names(&p.pattern))
                .collect();
            if let Some(expr) = arrow.body.as_expression() {
                return find_in_expr(ctx, expr, &params, ActionMode::Listing);
            }
            let stmts = crate::ast::arrow_function_body_statements(&arrow.body)?;
            find_in_stmts(ctx, stmts, &params, ActionMode::Listing)
        }
        Expression::FunctionExpression(func) => {
            let params: Vec<_> = func
                .params
                .items
                .iter()
                .flat_map(|p| pattern_names(&p.pattern))
                .collect();
            let stmts = &func.body.as_ref()?.statements;
            find_in_stmts(ctx, stmts, &params, ActionMode::Listing)
        }
        _ => None,
    }
}
