use super::*;
use oxc_ast::ast::{ArrayExpression, ArrayExpressionElement, ObjectExpression, ObjectPropertyKind};
pub(super) fn object(object: &ObjectExpression<'_>) -> Node {
    Node::Object(
        object
            .properties
            .iter()
            .map(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => (
                    property_key(&property.key).map_or(Field::Unknown, |key| {
                        if key == "__proto__" && !property.computed {
                            Field::Unknown
                        } else {
                            Field::Key(key.to_string())
                        }
                    }),
                    span(property.value.span()),
                ),
                ObjectPropertyKind::SpreadProperty(spread) => {
                    (Field::Spread, span(spread.argument.span()))
                }
            })
            .collect(),
    )
}
pub(super) fn array(array: &ArrayExpression<'_>) -> Node {
    let unknown = array
        .elements
        .iter()
        .any(|value| matches!(value, ArrayExpressionElement::SpreadElement(_)));
    Node::Object(
        array
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                if let ArrayExpressionElement::SpreadElement(value) = value {
                    Some((Field::Unknown, span(value.argument.span())))
                } else {
                    value.as_expression().map(|value| {
                        (
                            if unknown {
                                Field::Unknown
                            } else {
                                Field::Key(index.to_string())
                            },
                            span(value.span()),
                        )
                    })
                }
            })
            .collect(),
    )
}

pub(super) fn methods(tracker: &mut Tracker, object: &ObjectExpression<'_>) {
    if object.properties.iter().any(|property| {
        !matches!(property,
        ObjectPropertyKind::ObjectProperty(property) if property_key(&property.key).is_some())
    }) {
        return;
    }
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let key = (span(object.span), property_key(&property.key).unwrap());
        tracker.methods.remove(&key);
        if property.kind == oxc_ast::ast::PropertyKind::Init {
            if let Expression::FunctionExpression(function) = unwrap_ts_wrappers(&property.value) {
                tracker.methods.insert(key, span(function.span));
            }
        }
    }
}
