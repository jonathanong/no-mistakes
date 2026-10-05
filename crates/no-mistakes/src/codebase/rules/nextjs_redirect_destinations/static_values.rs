use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind};
use oxc_span::GetSpan;
use std::collections::BTreeMap;
use std::sync::Arc;

mod bindings;
mod functions;
pub(super) use bindings::{program_environment, scope_environment};
pub(super) use functions::parameter_environment;

#[derive(Clone, Debug)]
pub(super) enum Value {
    String(String),
    Array(Arc<Vec<Value>>),
    Object(Arc<BTreeMap<String, (Value, u32)>>, bool),
    Unknown,
}
pub(super) type Environment = BTreeMap<String, Value>;
pub(super) struct Evaluator {
    remaining: usize,
}
impl Evaluator {
    pub(super) fn new() -> Self {
        Self { remaining: 4096 }
    }
    pub(super) fn expression(
        &mut self,
        expr: &Expression<'_>,
        env: &Environment,
        depth: usize,
    ) -> Value {
        if self.remaining == 0 || depth > 64 {
            return Value::Unknown;
        }
        self.remaining -= 1;
        let depth = depth + 1;
        match unwrap_ts_wrappers(expr) {
            Expression::StringLiteral(literal) => Value::String(literal.value.to_string()),
            Expression::Identifier(ident) => env
                .get(ident.name.as_str())
                .cloned()
                .unwrap_or(Value::Unknown),
            Expression::TemplateLiteral(template) => {
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
            Expression::ArrayExpression(array) => {
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
            Expression::ObjectExpression(object) => {
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
            Expression::CallExpression(call) => self.map(call, env, depth),
            Expression::StaticMemberExpression(member) => {
                match self.expression(&member.object, env, depth) {
                    Value::Object(values, true) => values
                        .get(member.property.name.as_str())
                        .map(|(value, _)| value.clone())
                        .unwrap_or(Value::Unknown),
                    _ => Value::Unknown,
                }
            }
            Expression::ComputedMemberExpression(member) => {
                let Value::Array(values) = self.expression(&member.object, env, depth) else {
                    return Value::Unknown;
                };
                let Expression::NumericLiteral(index) = unwrap_ts_wrappers(&member.expression)
                else {
                    return Value::Unknown;
                };
                if index.value < 0.0 || index.value.fract() != 0.0 {
                    return Value::Unknown;
                }
                values
                    .get(index.value as usize)
                    .cloned()
                    .unwrap_or(Value::Unknown)
            }
            _ => Value::Unknown,
        }
    }
}
