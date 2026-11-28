use super::{ModeState, TtlState};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;

pub(super) fn fold_dedup_object(
    visitor: &QueueVisitor<'_>,
    object: &ObjectExpression<'_>,
    mode: &mut ModeState,
    ttl: &mut TtlState,
    active: &mut crate::fx::FxHashSet<u32>,
    budget: &mut u16,
) {
    for property in &object.properties {
        if *budget == 0 {
            *mode = ModeState::Dynamic;
            *ttl = TtlState::Dynamic;
            return;
        }
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some("mode") => *mode = super::eval_mode(visitor, &property.value),
                    Some("ttl") => *ttl = super::ttl::classify(visitor, &property.value),
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
                    budget,
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
    active: &mut crate::fx::FxHashSet<u32>,
    budget: &mut u16,
) {
    if *budget == 0 {
        *mode = ModeState::Dynamic;
        *ttl = TtlState::Dynamic;
        return;
    }
    let inner = expr.get_inner_expression();
    if noop_spread(visitor, inner) {
        return;
    }
    *budget -= 1;
    match inner {
        Expression::ObjectExpression(object) => {
            fold_dedup_object(visitor, object, mode, ttl, active, budget);
        }
        Expression::Identifier(id) => {
            let Some(binding) = visitor.object_bindings.lookup(id.name.as_str(), at) else {
                *mode = ModeState::Dynamic;
                *ttl = TtlState::Dynamic;
                return;
            };
            if binding
                .init
                .is_some_and(|init| noop_spread(visitor, init.get_inner_expression()))
            {
                return;
            }
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
            fold_spread(
                visitor,
                value,
                binding.resolve_at,
                mode,
                ttl,
                active,
                budget,
            );
            active.remove(&binding.resolve_at);
        }
        _ => {
            *mode = ModeState::Dynamic;
            *ttl = TtlState::Dynamic;
        }
    }
}

fn noop_spread(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::NullLiteral(_)) || super::unbound_undefined(visitor, expr)
}
