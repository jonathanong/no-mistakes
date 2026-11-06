use super::{Kind, Value};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
mod containers;
use oxc_span::GetSpan;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) struct Evaluator<'a, 'b> {
    pub(super) path: &'b Path,
    pub(super) source: &'b str,
    pub(super) bindings: BTreeMap<String, &'a Expression<'a>>,
    pub(super) imports: BTreeMap<String, Value>,
    pub(super) wrappers: BTreeSet<String>,
    pub(super) visiting: BTreeSet<String>,
}
impl Evaluator<'_, '_> {
    fn line(&self, offset: u32) -> usize {
        self.source[..offset as usize]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    }
    pub(super) fn named(&mut self, name: &str) -> Option<Value> {
        if let Some(value) = self.imports.get(name) {
            return Some(value.clone());
        }
        if !self.visiting.insert(name.to_string()) {
            return None;
        }
        let value = self
            .bindings
            .get(name)
            .copied()
            .map(|expression| self.eval(expression));
        self.visiting.remove(name);
        value
    }
    pub(super) fn eval(&mut self, expression: &Expression<'_>) -> Value {
        let span = expression.span();
        let kind = match unwrap_ts_wrappers(expression) {
            Expression::BooleanLiteral(value) => Kind::Literal(value.value.into()),
            Expression::NullLiteral(_) => Kind::Literal(serde_json::Value::Null),
            Expression::StringLiteral(value) => {
                Kind::Literal(crate::ast::js_literal(value).as_ref().into())
            }
            Expression::NumericLiteral(value) => serde_json::Number::from_f64(value.value)
                .map(|value| Kind::Literal(value.into()))
                .unwrap_or(Kind::Unknown),
            Expression::UnaryExpression(unary) => {
                let Expression::NumericLiteral(number) = unwrap_ts_wrappers(&unary.argument) else {
                    return Value {
                        kind: Kind::Unknown,
                        file: self.path.to_path_buf(),
                        line: self.line(span.start),
                    };
                };
                let number = match unary.operator {
                    oxc_ast::ast::UnaryOperator::UnaryNegation => Some(-number.value),
                    oxc_ast::ast::UnaryOperator::UnaryPlus => Some(number.value),
                    _ => None,
                };
                number
                    .and_then(serde_json::Number::from_f64)
                    .map(|number| Kind::Literal(number.into()))
                    .unwrap_or(Kind::Unknown)
            }
            Expression::Identifier(identifier) => {
                if let Some(value) = self.named(identifier.name.as_str()) {
                    return value;
                }
                Kind::Unknown
            }
            Expression::StaticMemberExpression(member) => {
                if let Some(value) = self
                    .eval(&member.object)
                    .property(member.property.name.as_str())
                {
                    return value;
                }
                Kind::Unknown
            }
            Expression::CallExpression(call) => {
                let recognized = matches!(&call.callee, Expression::Identifier(identifier)
                    if self.wrappers.contains(identifier.name.as_str()) && !self.bindings.contains_key(identifier.name.as_str()));
                if recognized && call.arguments.len() == 1 {
                    if let Some(expression) = call.arguments[0].as_expression() {
                        return self.eval(expression);
                    }
                }
                Kind::Unknown
            }
            Expression::ObjectExpression(object) => self.object(object),
            Expression::ArrayExpression(array) => self.array(array),
            _ => Kind::Unknown,
        };
        Value {
            kind,
            file: self.path.to_path_buf(),
            line: self.line(span.start),
        }
    }
}
