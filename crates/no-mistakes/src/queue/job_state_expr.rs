use super::extract::{ExtractCtx, FnEnv, StateMatch, TrackedJob};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::queue::extract_helpers::static_string;
use crate::queue::extract_model::ProducerBinding;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, CallExpression, ChainElement, Expression,
    Statement,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashMap;

pub(super) fn unwrap_expr<'a>(mut expr: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        expr = unwrap_ts_wrappers(expr);
        if let Expression::AwaitExpression(await_expr) = expr {
            expr = &await_expr.argument;
            continue;
        }
        return expr;
    }
}

pub(super) fn as_call<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match unwrap_expr(expr) {
        Expression::CallExpression(call) => Some(call),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => Some(call),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn call_member<'a>(
    call: &'a CallExpression<'a>,
) -> Option<(&'a Expression<'a>, &'a str)> {
    match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member) => {
            Some((&member.object, member.property.name.as_str()))
        }
        _ => None,
    }
}

pub(super) fn match_get_job(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
) -> Option<ProducerBinding> {
    let call = as_call(expr)?;
    let (object, method) = call_member(call)?;
    (method == "getJob").then(|| ctx.queue_receiver(object, call.span.start))?
}

pub(super) fn match_listing(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
) -> Option<ProducerBinding> {
    let call = as_call(expr)?;
    let (object, method) = call_member(call)?;
    let is_listing = match method {
        "getFailed" | "getCompleted" => true,
        "getJobs" => call.arguments.first().is_some_and(has_terminal_job_state),
        _ => false,
    };
    is_listing.then(|| ctx.queue_receiver(object, call.span.start))?
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
            matches!(method, "map" | "filter" | "flatMap" | "slice" | "concat")
                .then(|| derived_listing_binding(ctx, object, env))?
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

fn has_terminal_job_state(arg: &Argument<'_>) -> bool {
    let Some(expr) = arg.as_expression() else {
        return false;
    };
    match unwrap_ts_wrappers(expr) {
        Expression::ArrayExpression(array) => array.elements.iter().any(|el| match el {
            ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => false,
            other => other
                .as_expression()
                .and_then(static_string)
                .is_some_and(|s| matches!(s.as_str(), "failed" | "completed")),
        }),
        other => static_string(other).is_some_and(|s| matches!(s.as_str(), "failed" | "completed")),
    }
}

pub(super) fn match_state_call(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    job_vars: &HashMap<String, TrackedJob>,
) -> Option<StateMatch> {
    let call = as_call(expr)?;
    let (object, method) = call_member(call)?;
    if !matches!(method, "getState" | "isFailed" | "isCompleted") {
        return None;
    }
    if let Expression::Identifier(id) = unwrap_expr(object) {
        if let Some(tracked) = job_vars.get(id.name.as_str()) {
            return Some(StateMatch {
                job_id: Some(tracked.id),
                binding: tracked.binding.clone(),
            });
        }
    }
    match_get_job(ctx, object).map(|binding| StateMatch {
        job_id: None,
        binding,
    })
}

pub(super) fn check_state_expr(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    env: &FnEnv,
) -> Option<StateMatch> {
    struct StateFinder<'a, 'b, 'c> {
        ctx: &'a mut ExtractCtx<'b, 'c>,
        env: &'a FnEnv,
        found: Option<StateMatch>,
    }
    impl<'a> Visit<'a> for StateFinder<'_, '_, '_> {
        fn visit_expression(&mut self, expr: &Expression<'a>) {
            if self.found.is_some() {
                return;
            }
            if let Some(matched) = match_state_call(self.ctx, expr, &self.env.job_vars) {
                self.found = Some(matched);
                return;
            }
            if let Expression::Identifier(id) = expr {
                if let Some(matched) = self.env.state_vars.get(id.name.as_str()) {
                    self.found = Some(matched.clone());
                    return;
                }
            }
            walk::walk_expression(self, expr);
        }
    }
    let mut finder = StateFinder {
        ctx,
        env,
        found: None,
    };
    finder.visit_expression(expr);
    finder.found
}

pub(super) fn is_early_exit(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => block.body.last().is_some_and(is_early_exit),
        _ => false,
    }
}

pub(super) fn pattern_names(pattern: &BindingPattern<'_>) -> Vec<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => vec![id.name.to_string()],
        _ => Vec::new(),
    }
}
