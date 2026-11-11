use super::{expression_object, shared, Ctx};
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKind};

pub(super) fn sequence(value: &Expression<'_>, ctx: &Ctx<'_, '_>) -> Option<bool> {
    let Some(object) = expression_object(value, &ctx.bindings) else {
        return Some(true);
    };
    let mut concurrent = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            concurrent = Some(true);
            continue;
        };
        let name = shared::property_key_name(&property.key);
        if property.computed && name.is_none() {
            concurrent = Some(true);
        } else if name.as_deref() == Some("concurrent") {
            concurrent = Some(
                property.method
                    || property.kind != PropertyKind::Init
                    || !matches!(shared::expression_value(&property.value, &ctx.bindings),
                    Expression::BooleanLiteral(value) if !value.value),
            );
        }
    }
    concurrent
}
