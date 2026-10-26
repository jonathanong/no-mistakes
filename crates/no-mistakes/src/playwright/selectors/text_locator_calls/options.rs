use super::property_key_name;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind};

fn expression_bool(expression: &Expression<'_>) -> Option<bool> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::BooleanLiteral(literal) => Some(literal.value),
        _ => None,
    }
}

pub(super) enum BoolProperty {
    Missing,
    Value(bool),
    Unknown,
}

pub(super) fn object_bool_property(argument: &Argument<'_>, name: &str) -> BoolProperty {
    let Argument::ObjectExpression(object) = argument else {
        return BoolProperty::Missing;
    };
    let mut value = BoolProperty::Missing;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return BoolProperty::Unknown;
        };
        if property.computed
            || property.method
            || property_key_name(&property.key).as_deref() != Some(name)
        {
            if property.computed || property.method {
                return BoolProperty::Unknown;
            }
            continue;
        }
        if let Some(bool_value) = expression_bool(&property.value) {
            value = BoolProperty::Value(bool_value);
        } else {
            value = BoolProperty::Unknown;
        }
    }
    value
}

pub(super) fn object_has_unsupported_role_filters(argument: &Argument<'_>) -> bool {
    let Argument::ObjectExpression(object) = argument else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return true;
        };
        if property.computed || property.method {
            return true;
        }
        matches!(
            property_key_name(&property.key).as_deref(),
            Some(
                "checked"
                    | "selected"
                    | "pressed"
                    | "expanded"
                    | "disabled"
                    | "level"
                    | "description"
            )
        )
    })
}
