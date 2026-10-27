use super::{expression, Expr};
use oxc_ast::ast::{ObjectExpression, ObjectPropertyKind};

pub(super) fn object(value: &ObjectExpression<'_>, source: &str) -> Expr {
    let precise = value.properties.iter().all(|property| {
        matches!(property, ObjectPropertyKind::ObjectProperty(property)
            if !property.computed
                && !property.method
                && property.kind == oxc_ast::ast::PropertyKind::Init
                && crate::ast::property_key_name(&property.key).is_some_and(|name| name != "__proto__"))
    });
    if precise {
        let (keys, values) = value
            .properties
            .iter()
            .filter_map(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => Some((
                    crate::ast::property_key_name(&property.key)?.into_owned(),
                    expression(&property.value, source),
                )),
                ObjectPropertyKind::SpreadProperty(_) => None,
            })
            .unzip();
        return Expr::Record(values, keys);
    }
    Expr::Object(
        value
            .properties
            .iter()
            .map(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let value = expression(&property.value, source);
                    if property.computed {
                        // Computed keys run first, but do not become property values.
                        let key = property.key.as_expression().expect("computed property key");
                        Expr::Sequence(vec![
                            Expr::Discard(Box::new(expression(key, source))),
                            value,
                        ])
                    } else {
                        value
                    }
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    // Conservatively retain the spread object's property candidates.
                    Expr::Member(
                        Box::new(expression(&spread.argument, source)),
                        String::new(),
                    )
                }
            })
            .collect(),
    )
}
