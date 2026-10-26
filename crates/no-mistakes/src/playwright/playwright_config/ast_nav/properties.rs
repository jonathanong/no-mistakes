use crate::ast::js_literal;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey};

pub fn property_expression<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property.method {
            continue;
        }
        if property_key_name(&property.key).as_deref() == Some(name) {
            return Some(&property.value);
        }
    }
    None
}

fn property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(literal) => Some(js_literal(&literal).into_owned()),
        _ => None,
    }
}

pub fn project_objects<'a>(root: &'a ObjectExpression<'a>) -> Vec<&'a ObjectExpression<'a>> {
    let Some(Expression::ArrayExpression(projects)) = property_expression(root, "projects") else {
        return Vec::new();
    };
    projects
        .elements
        .iter()
        .filter_map(array_element_object)
        .collect()
}

fn array_element_object<'a>(
    element: &'a oxc_ast::ast::ArrayExpressionElement<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match element {
        oxc_ast::ast::ArrayExpressionElement::ObjectExpression(object) => Some(object),
        _ => None,
    }
}
