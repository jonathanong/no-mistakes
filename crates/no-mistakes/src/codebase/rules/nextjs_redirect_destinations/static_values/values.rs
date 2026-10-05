use super::*;
impl Evaluator {
    pub(super) fn template(
        &mut self,
        template: &oxc_ast::ast::TemplateLiteral<'_>,
        env: &Environment,
        depth: usize,
    ) -> Value {
        let mut value = String::new();
        for (index, quasi) in template.quasis.iter().enumerate() {
            let Some(text) = quasi.value.cooked.as_ref() else {
                return Value::Unknown;
            };
            if value.len().saturating_add(text.len()) > 65536 {
                return Value::Unknown;
            }
            value.push_str(text.as_str());
            if let Some(expr) = template.expressions.get(index) {
                let Value::String(part) = self.expression(expr, env, depth) else {
                    return Value::Unknown;
                };
                if value.len().saturating_add(part.len()) > 65536 {
                    return Value::Unknown;
                }
                value.push_str(&part);
            }
        }
        Value::String(value)
    }
    pub(super) fn array(
        &mut self,
        array: &oxc_ast::ast::ArrayExpression<'_>,
        env: &Environment,
        depth: usize,
    ) -> Value {
        let mut values = Vec::new();
        for element in &array.elements {
            if self.remaining == 0 || values.len() > 4096 {
                values.push(Value::Unknown);
                break;
            }
            if let ArrayExpressionElement::SpreadElement(spread) = element {
                match self.expression(&spread.argument, env, depth) {
                    Value::Array(spread) => values.extend(spread.iter().cloned()),
                    _ => values.push(Value::Unknown),
                }
            } else if let Some(expr) = element.as_expression() {
                values.push(self.expression(expr, env, depth));
            } else {
                values.push(Value::Unknown);
            }
        }
        Value::Array(Arc::new(values))
    }
    pub(super) fn object(
        &mut self,
        object: &oxc_ast::ast::ObjectExpression<'_>,
        env: &Environment,
        depth: usize,
    ) -> Value {
        let mut properties = BTreeMap::new();
        let mut complete = true;
        for property in &object.properties {
            if self.remaining == 0 {
                complete = false;
                break;
            }
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    if property.computed || property.method {
                        complete = false;
                        continue;
                    }
                    let Some(key) = static_property_key_name(&property.key) else {
                        complete = false;
                        continue;
                    };
                    properties.insert(
                        key.to_string(),
                        (
                            self.expression(&property.value, env, depth),
                            property.value.span().start,
                        ),
                    );
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    match self.expression(&spread.argument, env, depth) {
                        Value::Object(values, known) => {
                            properties.extend(
                                values
                                    .iter()
                                    .map(|(key, value)| (key.clone(), value.clone())),
                            );
                            complete &= known;
                        }
                        _ => {
                            complete = false;
                            properties.clear();
                        }
                    }
                }
            }
        }
        Value::Object(Arc::new(properties), complete)
    }
}
