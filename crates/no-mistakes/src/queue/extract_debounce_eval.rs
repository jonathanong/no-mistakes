use super::members::extract_member_from_expr;
use super::DebounceTtlVerdict;
use crate::fx::FxHashSet;
use crate::queue::extract_helpers::literal_expr;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{Expression, UnaryOperator};
use oxc_span::GetSpan;

#[path = "extract_debounce_eval_fold.rs"]
mod fold;
#[path = "extract_debounce_eval_ttl.rs"]
mod ttl;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModeState {
    Absent,
    Debounce,
    Other,
    Dynamic,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TtlState {
    Absent,
    Present,
    Dynamic,
}

pub(super) fn eval_options<'a>(
    visitor: &QueueVisitor<'a>,
    expr: &Expression<'a>,
    at: u32,
    active: &mut FxHashSet<u32>,
) -> DebounceTtlVerdict {
    match extract_member_from_expr(
        expr,
        at,
        "deduplication",
        &visitor.object_bindings,
        active,
        &mut 32,
    ) {
        super::members::MemberLookup::Absent => DebounceTtlVerdict::Clean,
        super::members::MemberLookup::Unknown => DebounceTtlVerdict::Abstain,
        super::members::MemberLookup::Found(dedup) => eval_dedup_expr(
            visitor,
            dedup,
            dedup.span().start,
            &mut FxHashSet::default(),
        ),
    }
}

pub(super) fn eval_dedup_expr(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    at: u32,
    active: &mut FxHashSet<u32>,
) -> DebounceTtlVerdict {
    let inner = expr.get_inner_expression();
    if is_non_object_literal_or_token(visitor, inner) {
        return DebounceTtlVerdict::Clean;
    }
    match inner {
        Expression::ObjectExpression(object) => {
            let mut mode = ModeState::Absent;
            let mut ttl = TtlState::Absent;
            let mut budget = 32u16;
            fold::fold_dedup_object(visitor, object, &mut mode, &mut ttl, active, &mut budget);
            verdict(mode, ttl)
        }
        Expression::Identifier(id) => {
            let Some(binding) = visitor.object_bindings.lookup(id.name.as_str(), at) else {
                return DebounceTtlVerdict::Abstain;
            };
            let Some(value) = binding.value() else {
                return DebounceTtlVerdict::Abstain;
            };
            if !active.insert(binding.resolve_at) {
                return DebounceTtlVerdict::Abstain;
            }
            let out = eval_dedup_expr(visitor, value, binding.resolve_at, active);
            active.remove(&binding.resolve_at);
            out
        }
        _ => DebounceTtlVerdict::Abstain,
    }
}

fn eval_mode(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> ModeState {
    let inner = expr.get_inner_expression();
    if unbound_undefined(visitor, inner) || matches!(inner, Expression::NullLiteral(_)) {
        return ModeState::Absent;
    }
    match literal_expr(inner, &visitor.const_strings).as_deref() {
        Some("debounce") => ModeState::Debounce,
        Some(_) => ModeState::Other,
        None => ModeState::Dynamic,
    }
}

pub(super) fn unbound_undefined(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(id) if id.name == "undefined" => {
            crate::queue::extract_job_id::referenced_symbol(visitor, id).is_none()
        }
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::Void,
        _ => false,
    }
}

fn is_non_object_literal_or_token(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> bool {
    visitor.tracks_job_token(expr)
        || unbound_undefined(visitor, expr)
        || matches!(
            expr,
            Expression::StringLiteral(_)
                | Expression::TemplateLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::NullLiteral(_)
        )
}

fn verdict(mode: ModeState, ttl: TtlState) -> DebounceTtlVerdict {
    match (mode, ttl) {
        (ModeState::Debounce, TtlState::Present) => DebounceTtlVerdict::Finding,
        (ModeState::Debounce, TtlState::Dynamic)
        | (ModeState::Dynamic, TtlState::Present | TtlState::Dynamic) => {
            DebounceTtlVerdict::Abstain
        }
        _ => DebounceTtlVerdict::Clean,
    }
}
