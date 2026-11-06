use super::{Evaluator, Kind, Value};
use oxc_ast::ast::{ArrayExpression, ArrayExpressionElement, ObjectExpression, ObjectPropertyKind};
use std::collections::BTreeMap;

impl Evaluator<'_, '_> {
    pub(super) fn object(&mut self, object: &ObjectExpression<'_>) -> Kind {
        let mut properties = BTreeMap::new();
        let mut opaque = false;
        for property in &object.properties {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let name = crate::integration_tests::test_config::shared::property_key_name(
                        &property.key,
                    );
                    if property.computed || property.method || name.is_none() {
                        properties.clear();
                        opaque = true;
                        continue;
                    }
                    properties.insert(name.unwrap(), self.eval(&property.value));
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    match self.eval(&spread.argument).kind {
                        Kind::Object(fields, unknown) => {
                            if unknown {
                                properties.clear();
                                opaque = true;
                            }
                            properties.extend(fields);
                        }
                        _ => {
                            properties.clear();
                            opaque = true;
                        }
                    }
                }
            }
        }
        Kind::Object(properties, opaque)
    }

    pub(super) fn array(&mut self, array: &ArrayExpression<'_>) -> Kind {
        let mut elements = Vec::new();
        for element in &array.elements {
            match element {
                ArrayExpressionElement::SpreadElement(spread) => {
                    match self.eval(&spread.argument).kind {
                        Kind::Array(values) => elements.extend(values),
                        _ => elements.push(self.eval(&spread.argument).unknown()),
                    }
                }
                ArrayExpressionElement::Elision(elision) => elements.push(Value {
                    kind: Kind::Unknown,
                    file: self.path.to_path_buf(),
                    line: self.line(elision.span.start),
                }),
                _ => {
                    if let Some(expression) = element.as_expression() {
                        elements.push(self.eval(expression));
                    }
                }
            }
        }
        Kind::Array(elements)
    }
}
