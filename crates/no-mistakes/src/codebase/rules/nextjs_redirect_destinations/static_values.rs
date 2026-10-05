use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind};
use oxc_span::GetSpan;
use std::collections::BTreeMap;
use std::sync::Arc;

mod bindings;
mod functions;
mod values;
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
            Expression::TemplateLiteral(template) => self.template(template, env, depth),
            Expression::ArrayExpression(array) => self.array(array, env, depth),
            Expression::ObjectExpression(object) => self.object(object, env, depth),
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
