use super::members::extract_member_from_expr;
use super::DebounceTtlVerdict;
use crate::fx::FxHashSet;
use crate::queue::extract_helpers::literal_expr;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, UnaryOperator};
use oxc_span::GetSpan;

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
    match extract_member_from_expr(expr, at, "deduplication", &visitor.object_bindings, active) {
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
            fold_dedup_object(visitor, object, &mut mode, &mut ttl, active);
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

fn fold_dedup_object(
    visitor: &QueueVisitor<'_>,
    object: &ObjectExpression<'_>,
    mode: &mut ModeState,
    ttl: &mut TtlState,
    active: &mut FxHashSet<u32>,
) {
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some("mode") => *mode = eval_mode(visitor, &property.value),
                    Some("ttl") => *ttl = ttl::classify(visitor, &property.value),
                    Some(_) => {}
                    None => {
                        *mode = ModeState::Dynamic;
                        *ttl = TtlState::Dynamic;
                    }
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                fold_spread(
                    visitor,
                    &spread.argument,
                    spread.argument.span().start,
                    mode,
                    ttl,
                    active,
                );
            }
        }
    }
}

fn fold_spread(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    at: u32,
    mode: &mut ModeState,
    ttl: &mut TtlState,
    active: &mut FxHashSet<u32>,
) {
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => {
            fold_dedup_object(visitor, object, mode, ttl, active)
        }
        Expression::Identifier(id) => {
            let Some(binding) = visitor.object_bindings.lookup(id.name.as_str(), at) else {
                *mode = ModeState::Dynamic;
                *ttl = TtlState::Dynamic;
                return;
            };
            let Some(value) = binding.value() else {
                *mode = ModeState::Dynamic;
                *ttl = TtlState::Dynamic;
                return;
            };
            if !active.insert(binding.resolve_at) {
                *mode = ModeState::Dynamic;
                *ttl = TtlState::Dynamic;
                return;
            }
            fold_spread(visitor, value, binding.resolve_at, mode, ttl, active);
            active.remove(&binding.resolve_at);
        }
        _ => {
            *mode = ModeState::Dynamic;
            *ttl = TtlState::Dynamic;
        }
    }
}

fn eval_mode(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> ModeState {
    let inner = expr.get_inner_expression();
    if is_undefined_expr(inner) || matches!(inner, Expression::NullLiteral(_)) {
        return ModeState::Absent;
    }
    match literal_expr(inner, &visitor.const_strings).as_deref() {
        Some("debounce") => ModeState::Debounce,
        Some(_) => ModeState::Other,
        None => ModeState::Dynamic,
    }
}

fn is_undefined_expr(expr: &Expression<'_>) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(id) => id.name == "undefined",
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::Void,
        _ => false,
    }
}

fn is_non_object_literal_or_token(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> bool {
    visitor.tracks_job_token(expr)
        || is_undefined_expr(expr)
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
