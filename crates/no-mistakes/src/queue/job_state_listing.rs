use super::actions::{find_in_expr, find_in_stmts, ActionMode};
use super::expr::{
    arg_expr, call_member, check_state_expr, match_listing, pattern_names, unwrap_expr,
};
use super::extract::{ExtractCtx, FnEnv};
use super::JobStatePattern;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::queue::extract_model::ProducerBinding;
use oxc_ast::ast::{
    ArrayExpressionElement, BindingPattern, CallExpression, ConditionalExpression, Expression,
    FormalParameters,
};
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
            let action = find_in_expr(self.ctx, &cond.consequent, &names, &names, mode)
                .or_else(|| find_in_expr(self.ctx, &cond.alternate, &names, &names, mode));
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
                        arg_expr(arg).and_then(|e| expr_uses_listing(self.ctx, e, self.env))
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

pub(super) fn derived_listing_binding(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    env: &FnEnv,
) -> Option<ProducerBinding> {
    if let Some(binding) = match_listing(ctx, expr) {
        return Some(binding);
    }
    match unwrap_expr(expr) {
        Expression::Identifier(id) => env.listing_vars.get(id.name.as_str()).cloned(),
        Expression::CallExpression(call) => {
            let (object, method) = call_member(call)?;
            if !matches!(method, "map" | "filter" | "flatMap" | "slice" | "concat") {
                return None;
            }
            derived_listing_binding(ctx, object, env).or_else(|| {
                (method == "concat").then(|| {
                    call.arguments
                        .iter()
                        .find_map(|arg| derived_listing_binding(ctx, arg_expr(arg)?, env))
                })?
            })
        }
        Expression::ArrayExpression(array) => array.elements.iter().find_map(|el| match el {
            ArrayExpressionElement::SpreadElement(spread) => {
                derived_listing_binding(ctx, &spread.argument, env)
            }
            _ => None,
        }),
        _ => None,
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
            let (jobs, args) = callback_bindings(&arrow.params);
            if let Some(expr) = arrow.body.as_expression() {
                return find_in_expr(ctx, expr, &jobs, &args, ActionMode::Listing);
            }
            let stmts = crate::ast::arrow_function_body_statements(&arrow.body)?;
            find_in_stmts(ctx, stmts, &jobs, &args, ActionMode::Listing)
        }
        Expression::FunctionExpression(func) => {
            let (jobs, args) = callback_bindings(&func.params);
            let stmts = &func.body.as_ref()?.statements;
            find_in_stmts(ctx, stmts, &jobs, &args, ActionMode::Listing)
        }
        _ => None,
    }
}

fn callback_bindings(params: &FormalParameters<'_>) -> (Vec<String>, Vec<String>) {
    let mut jobs = Vec::new();
    let mut args = Vec::new();
    if let Some(first) = params.items.first() {
        let names = pattern_names(&first.pattern);
        if matches!(first.pattern, BindingPattern::BindingIdentifier(_)) {
            jobs.extend(names.iter().cloned());
        }
        args.extend(names);
    }
    if let Some(third) = params.items.get(2) {
        args.extend(pattern_names(&third.pattern));
    }
    (jobs, args)
}
