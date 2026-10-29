use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKind};

pub(super) fn is_closed_literal(expression: &Expression<'_>) -> bool {
    closed_literal(expression, 16)
}

fn closed_literal(expression: &Expression<'_>, remaining: usize) -> bool {
    if remaining == 0 {
        return false;
    }
    let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
    match expression {
        Expression::ObjectExpression(object) => object.properties.iter().all(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            property.kind == PropertyKind::Init
                && !property.method
                && !property.shorthand
                && !property.computed
                && closed_literal(&property.value, remaining - 1)
        }),
        Expression::ArrayExpression(array) => array.elements.iter().all(|element| {
            element
                .as_expression()
                .is_some_and(|value| closed_literal(value, remaining - 1))
        }),
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        other => other.is_literal(),
    }
}
