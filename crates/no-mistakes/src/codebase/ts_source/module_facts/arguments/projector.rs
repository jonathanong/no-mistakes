struct Projector<'a, 's> {
    semantic: &'s Semantic<'a>,
    call_offset: u32,
    remaining: usize,
    exported: &'s FxHashSet<usize>,
    immutable: &'s mut AliasSafety,
}

impl Projector<'_, '_> {
    fn take(&mut self) -> bool {
        if self.remaining == 0 {
            return false;
        }
        self.remaining -= 1;
        true
    }

    fn value(&mut self, expression: &Expression<'_>, depth: usize) -> ArgumentValue {
        if depth >= 16 || !self.take() {
            return ArgumentValue::Unknown;
        }
        match expression.get_inner_expression() {
            Expression::StringLiteral(value) if value.value.len() <= 4096 => {
                ArgumentValue::Scalar(Value::String(value.value.to_string()))
            }
            Expression::NumericLiteral(value) => serde_json::Number::from_f64(value.value)
                .map_or(ArgumentValue::Unknown, |value| {
                    ArgumentValue::Scalar(Value::Number(value))
                }),
            Expression::BooleanLiteral(value) => ArgumentValue::Scalar(Value::Bool(value.value)),
            Expression::NullLiteral(_) => ArgumentValue::Scalar(Value::Null),
            Expression::UnaryExpression(unary) => {
                let Expression::NumericLiteral(number) = unary.argument.get_inner_expression()
                else {
                    return ArgumentValue::Unknown;
                };
                let value = match unary.operator {
                    oxc_syntax::operator::UnaryOperator::UnaryNegation => -number.value,
                    oxc_syntax::operator::UnaryOperator::UnaryPlus => number.value,
                    _ => return ArgumentValue::Unknown,
                };
                serde_json::Number::from_f64(value).map_or(ArgumentValue::Unknown, |value| {
                    ArgumentValue::Scalar(Value::Number(value))
                })
            }
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
                .quasis
                .first()
                .and_then(|part| part.value.cooked.as_ref())
                .filter(|text| text.len() <= 4096)
                .map_or(ArgumentValue::Unknown, |text| {
                    ArgumentValue::Scalar(Value::String(text.to_string()))
                }),
            Expression::Identifier(identifier) => self.named(identifier, depth),
            Expression::ArrayExpression(array) if array.elements.len() <= 256 => {
                let mut values = Vec::with_capacity(array.elements.len());
                for element in &array.elements {
                    let Some(expression) = element.as_expression() else {
                        return ArgumentValue::Unknown;
                    };
                    values.push(self.value(expression, depth + 1));
                }
                ArgumentValue::Array(values)
            }
            Expression::ObjectExpression(object) if object.properties.len() <= 256 => {
                let mut values = FxHashMap::default();
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return ArgumentValue::Unknown;
                    };
                    if property.kind != PropertyKind::Init || property.method || property.computed {
                        return ArgumentValue::Unknown;
                    }
                    let Some(key) = property.key.static_name() else {
                        return ArgumentValue::Unknown;
                    };
                    if key == "__proto__" || key.len() > 128 {
                        return ArgumentValue::Unknown;
                    }
                    values.insert(key.to_string(), self.value(&property.value, depth + 1));
                }
                ArgumentValue::Object(values)
            }
            _ => ArgumentValue::Unknown,
        }
    }
}
