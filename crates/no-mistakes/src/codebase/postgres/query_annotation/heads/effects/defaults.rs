//! Inert later parameter defaults and closed caller objects for prefix effects.
use super::Signature;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, ObjectPropertyKind, PropertyKind};

pub(super) fn plain_parameter(
    index: usize,
    pattern: &BindingPattern<'_>,
    initializer: Option<&Expression<'_>>,
    signature: &mut Signature,
) -> bool {
    if index == 0 && initializer.is_some() {
        return false;
    }
    match pattern {
        BindingPattern::BindingIdentifier(_) => initializer.is_none_or(literal_default),
        BindingPattern::ObjectPattern(object) if index > 0 && object.rest.is_none() => {
            if initializer.is_some_and(|value| !empty_object(value)) {
                return false;
            }
            let mut property_default = false;
            let valid = object.properties.iter().all(|property| {
                !property.computed
                    && crate::ast::property_key_name(&property.key)
                        .is_some_and(|name| name != "__proto__")
                    && match &property.value {
                        BindingPattern::BindingIdentifier(_) => true,
                        BindingPattern::AssignmentPattern(default) => {
                            property_default = true;
                            matches!(default.left, BindingPattern::BindingIdentifier(_))
                                && literal_default(&default.right)
                        }
                        _ => false,
                    }
            });
            if valid && (property_default || initializer.is_some()) {
                if index >= 16 {
                    return false;
                }
                signature.required_objects |= 1 << index;
                if initializer.is_some() {
                    signature.empty_defaults |= 1 << index;
                }
            }
            valid
        }
        _ => false,
    }
}

fn literal_default(value: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(value),
        Expression::BooleanLiteral(_) | Expression::StringLiteral(_)
    )
}

fn empty_object(value: &Expression<'_>) -> bool {
    matches!(unwrap_ts_wrappers(value), Expression::ObjectExpression(object) if object.properties.is_empty())
}

pub(super) fn closed_object(value: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = unwrap_ts_wrappers(value) else {
        return false;
    };
    object.properties.iter().all(|entry| match entry {
        ObjectPropertyKind::ObjectProperty(property) => {
            !property.computed
                && !property.method
                && property.kind == PropertyKind::Init
                && crate::ast::property_key_name(&property.key)
                    .is_some_and(|name| name != "__proto__")
                && matches!(
                    unwrap_ts_wrappers(&property.value),
                    Expression::BooleanLiteral(_)
                        | Expression::StringLiteral(_)
                        | Expression::NumericLiteral(_)
                        | Expression::NullLiteral(_)
                )
        }
        ObjectPropertyKind::SpreadProperty(_) => false,
    })
}
