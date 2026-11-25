use crate::fx::FxHashSet;
use crate::queue::extract_helpers::literal_expr;
use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, ObjectExpression,
};
use oxc_span::GetSpan;

#[path = "extract_debounce_eval.rs"]
mod eval;
#[path = "extract_debounce_members.rs"]
mod members;

pub(in crate::queue) const DEBOUNCE_TTL_KIND: &str = "debounce-ttl";
pub(in crate::queue) const DEBOUNCE_TTL_MESSAGE: &str =
    "deduplication ttl is ignored when mode is debounce";
pub(in crate::queue) const DEBOUNCE_TTL_ABSTENTION: &str =
    "debounce-ttl was not judged because deduplication options are not statically known";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::queue) enum DebounceTtlVerdict {
    Clean,
    Finding,
    Abstain,
}

pub(in crate::queue) fn check_options_expr<'a>(
    visitor: &mut QueueVisitor<'a>,
    expr: &Expression<'a>,
    line: usize,
    job: Option<String>,
) {
    let verdict = eval::eval_options(visitor, expr, expr.span().start, &mut FxHashSet::default());
    emit_verdict(visitor, verdict, line, job);
}

pub(in crate::queue) fn check_dedup_value<'a>(
    visitor: &QueueVisitor<'a>,
    expr: &Expression<'a>,
) -> DebounceTtlVerdict {
    eval::eval_dedup_expr(visitor, expr, expr.span().start, &mut FxHashSet::default())
}

pub(in crate::queue) fn check_bulk_call<'a>(
    visitor: &mut QueueVisitor<'a>,
    call: &CallExpression<'a>,
) {
    let Some(Expression::ArrayExpression(array)) = call
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        return;
    };
    for element in &array.elements {
        let ArrayExpressionElement::ObjectExpression(object) = element else {
            continue;
        };
        let job = members::extract_member(
            object,
            "name",
            &visitor.object_bindings,
            &mut FxHashSet::default(),
        )
        .and_then(|expr| literal_expr(expr.get_inner_expression(), &visitor.const_strings));
        check_opts_member(
            visitor,
            object,
            line_number(visitor.source, object.span.start),
            job,
        );
    }
}

pub(in crate::queue) fn check_flow_node<'a>(
    visitor: &mut QueueVisitor<'a>,
    object: &ObjectExpression<'a>,
    job: Option<String>,
) {
    check_opts_member(
        visitor,
        object,
        line_number(visitor.source, object.span.start),
        job,
    );
}

pub(in crate::queue) fn check_queue_options<'a>(
    visitor: &mut QueueVisitor<'a>,
    expr: &Expression<'a>,
    line: usize,
) {
    let Some(defaults) = members::extract_member_from_expr(
        expr,
        expr.span().start,
        "defaultJobOptions",
        &visitor.object_bindings,
        &mut FxHashSet::default(),
    ) else {
        return;
    };
    let verdict = eval::eval_options(
        visitor,
        defaults,
        defaults.span().start,
        &mut FxHashSet::default(),
    );
    emit_verdict(visitor, verdict, line, None);
}

fn check_opts_member<'a>(
    visitor: &mut QueueVisitor<'a>,
    object: &ObjectExpression<'a>,
    line: usize,
    job: Option<String>,
) {
    let Some(opts) = members::extract_member(
        object,
        "opts",
        &visitor.object_bindings,
        &mut FxHashSet::default(),
    ) else {
        return;
    };
    let verdict = eval::eval_options(visitor, opts, opts.span().start, &mut FxHashSet::default());
    emit_verdict(visitor, verdict, line, job);
}

pub(in crate::queue) fn emit_verdict(
    visitor: &mut QueueVisitor<'_>,
    verdict: DebounceTtlVerdict,
    line: usize,
    job: Option<String>,
) {
    match verdict {
        DebounceTtlVerdict::Clean => {}
        DebounceTtlVerdict::Finding => {
            visitor.facts.checks.push(LocalCheck {
                line,
                kind: DEBOUNCE_TTL_KIND.to_string(),
                job,
                message: DEBOUNCE_TTL_MESSAGE.to_string(),
            });
        }
        DebounceTtlVerdict::Abstain => {
            visitor
                .facts
                .check_abstentions
                .push((line, DEBOUNCE_TTL_ABSTENTION));
        }
    }
}
