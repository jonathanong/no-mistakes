use super::super::extract_model::{RetentionEvidence, RetentionValue};
use super::super::object_bindings::ObjectBindings;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::config::v2::schema::JobRetentionKey;
use crate::fx::FxHashSet;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};

pub(in crate::queue) fn from_expr(
    expr: &Expression<'_>,
    bindings: &ObjectBindings<'_>,
) -> RetentionEvidence {
    let mut active = FxHashSet::default();
    object_retention(expr, bindings, &mut active)
}

pub(in crate::queue) fn from_object(
    object: &ObjectExpression<'_>,
    bindings: &ObjectBindings<'_>,
) -> RetentionEvidence {
    let mut active = FxHashSet::default();
    read_object_retention(object, bindings, &mut active)
}

fn object_retention(
    expr: &Expression<'_>,
    bindings: &ObjectBindings<'_>,
    active: &mut FxHashSet<u32>,
) -> RetentionEvidence {
    match unwrap_ts_wrappers(expr) {
        Expression::ObjectExpression(object) => read_object_retention(object, bindings, active),
        Expression::Identifier(id) => {
            let Some(binding) = bindings.lookup(id.name.as_str(), id.span.start) else {
                return RetentionEvidence::unknown_all();
            };
            let Some(value) = binding.value() else {
                return RetentionEvidence::unknown_all();
            };
            if !active.insert(binding.resolve_at) {
                return RetentionEvidence::unknown_all();
            }
            let out = object_retention(value, bindings, active);
            active.remove(&binding.resolve_at);
            out
        }
        _ => RetentionEvidence::unknown_all(),
    }
}

fn read_object_retention(
    object: &ObjectExpression<'_>,
    bindings: &ObjectBindings<'_>,
    active: &mut FxHashSet<u32>,
) -> RetentionEvidence {
    let mut out = RetentionEvidence::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some("removeOnComplete") => {
                        let value = classify_value(&property.value, bindings, active);
                        out.set(JobRetentionKey::RemoveOnComplete, value);
                    }
                    Some("removeOnFail") => {
                        let value = classify_value(&property.value, bindings, active);
                        out.set(JobRetentionKey::RemoveOnFail, value);
                    }
                    Some(_) => {}
                    None => out.invalidate_unset(),
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let spread_ret = object_retention(&spread.argument, bindings, active);
                out.absorb(&spread_ret, false);
            }
        }
    }
    out
}

fn classify_value(
    expr: &Expression<'_>,
    bindings: &ObjectBindings<'_>,
    active: &mut FxHashSet<u32>,
) -> RetentionValue {
    match unwrap_ts_wrappers(expr) {
        Expression::BooleanLiteral(literal) => {
            if literal.value {
                RetentionValue::Bounded
            } else {
                RetentionValue::Unbounded
            }
        }
        Expression::NumericLiteral(_) => RetentionValue::Bounded,
        Expression::ObjectExpression(object) => classify_keep_object(object, bindings, active),
        Expression::Identifier(id) => with_const_init(id, bindings, active, classify_value),
        _ => RetentionValue::Unknown,
    }
}

fn classify_keep_object(
    object: &ObjectExpression<'_>,
    bindings: &ObjectBindings<'_>,
    active: &mut FxHashSet<u32>,
) -> RetentionValue {
    let mut has_bound = false;
    let mut has_unknown = false;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some("count" | "age") => {
                        if !is_explicitly_absent(&property.value) {
                            has_bound = true;
                        }
                    }
                    Some(_) => {}
                    None => has_unknown = true,
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                match classify_spread_keep(&spread.argument, bindings, active) {
                    RetentionValue::Bounded => has_bound = true,
                    RetentionValue::Unknown => has_unknown = true,
                    RetentionValue::Unbounded => {}
                }
            }
        }
    }
    if has_bound {
        RetentionValue::Bounded
    } else if has_unknown {
        RetentionValue::Unknown
    } else {
        RetentionValue::Unbounded
    }
}

fn classify_spread_keep(
    expr: &Expression<'_>,
    bindings: &ObjectBindings<'_>,
    active: &mut FxHashSet<u32>,
) -> RetentionValue {
    match unwrap_ts_wrappers(expr) {
        Expression::ObjectExpression(object) => classify_keep_object(object, bindings, active),
        Expression::Identifier(id) => with_const_init(id, bindings, active, classify_spread_keep),
        _ => RetentionValue::Unknown,
    }
}

fn with_const_init(
    id: &oxc_ast::ast::IdentifierReference<'_>,
    bindings: &ObjectBindings<'_>,
    active: &mut FxHashSet<u32>,
    classify: fn(&Expression<'_>, &ObjectBindings<'_>, &mut FxHashSet<u32>) -> RetentionValue,
) -> RetentionValue {
    let Some(binding) = bindings.lookup(id.name.as_str(), id.span.start) else {
        return RetentionValue::Unknown;
    };
    let Some(value) = binding.init else {
        return RetentionValue::Unknown;
    };
    if !active.insert(binding.resolve_at) {
        return RetentionValue::Unknown;
    }
    let result = classify(value, bindings, active);
    active.remove(&binding.resolve_at);
    result
}

fn is_explicitly_absent(expr: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::NullLiteral(_) => true,
        Expression::Identifier(id) => id.name == "undefined",
        _ => false,
    }
}
