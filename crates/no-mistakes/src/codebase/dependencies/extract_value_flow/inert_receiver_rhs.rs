use super::*;

// This gate admits only supported inert construction syntax. It does not prove
// arbitrary JavaScript purity or evaluate getters, spreads, calls, or classes.
pub(super) fn inert_rhs(
    expression: &Expression<'_>,
    effects: &mut fresh_receiver_effects::FreshReceiverEffects,
) -> bool {
    if matches!(
        crate::codebase::ts_source::unwrap_ts_wrappers(expression),
        Expression::Identifier(_)
    ) {
        effects.note_rhs_visit();
        return false;
    }
    let mut pending = vec![expression];
    while let Some(expression) = pending.pop() {
        effects.note_rhs_visit();
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::Identifier(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_) => {}
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    effects.note_rhs_visit();
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return false;
                    };
                    if property.computed
                        || property.kind != oxc_ast::ast::PropertyKind::Init
                        || crate::codebase::ts_source::static_property_key_name(&property.key)
                            == Some("__proto__")
                    {
                        return false;
                    }
                    pending.push(&property.value);
                }
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    effects.note_rhs_visit();
                    if matches!(element, oxc_ast::ast::ArrayExpressionElement::Elision(_)) {
                        continue;
                    }
                    let Some(value) = element.as_expression() else {
                        return false;
                    };
                    pending.push(value);
                }
            }
            _ => return false,
        }
    }
    true
}
