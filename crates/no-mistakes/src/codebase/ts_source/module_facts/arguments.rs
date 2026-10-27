//! Bounded invocation-local literal facts; never evaluate source code.
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use serde_json::Value;

mod aliases;
use aliases::AliasSafety;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ArgumentValue {
    Scalar(Value),
    Array(Vec<Self>),
    Object(FxHashMap<String, Self>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArgumentValues {
    pub(crate) values: Vec<ArgumentValue>,
    pub(crate) spread_from: Option<usize>,
}

pub(super) fn collect(
    semantic: &Semantic<'_>,
    facts: &super::TypeScriptModuleFacts,
) -> FxHashMap<u32, ArgumentValues> {
    let names: FxHashSet<_> = facts
        .exports
        .iter()
        .filter(|e| e.specifier.is_none() && !e.type_only)
        .map(|e| e.local.as_str())
        .collect();
    let roots: FxHashSet<_> = facts
        .scopes
        .iter()
        .filter(|s| s.parent_id.is_none())
        .map(|s| s.id)
        .collect();
    let exported: FxHashSet<_> = facts
        .bindings
        .iter()
        .filter(|b| roots.contains(&b.scope_id) && names.contains(b.name.as_str()))
        .map(|b| b.id)
        .collect();
    let mut immutable = AliasSafety::default();
    let mut result = FxHashMap::default();
    let mut dynamic_scope = false;
    for node in semantic.nodes().iter() {
        if matches!(node.kind(), AstKind::WithStatement(_)) {
            dynamic_scope = true;
        }
        if let AstKind::CallExpression(call) = node.kind() {
            if let Expression::Identifier(id) = call.callee.get_inner_expression() {
                if id.name == "eval"
                    && id
                        .reference_id
                        .get()
                        .is_some_and(|r| semantic.scoping().get_reference(r).symbol_id().is_none())
                {
                    dynamic_scope = true;
                }
            }
        }
        let (offset, arguments) = match node.kind() {
            AstKind::CallExpression(call) => (call.span.start, &call.arguments),
            AstKind::NewExpression(call) => (call.span.start, &call.arguments),
            _ => continue,
        };
        let mut projector = Projector {
            semantic,
            call_offset: offset,
            remaining: 256,
            exported: &exported,
            immutable: &mut immutable,
        };
        let mut spread = false;
        let spread_from = arguments
            .iter()
            .position(|argument| argument.as_expression().is_none());
        let values = arguments
            .iter()
            .take(32)
            .map(|argument| {
                let Some(expression) = argument.as_expression() else {
                    spread = true;
                    return ArgumentValue::Unknown;
                };
                if spread {
                    ArgumentValue::Unknown
                } else {
                    projector.value(expression, 0)
                }
            })
            .collect();
        result.insert(
            offset,
            ArgumentValues {
                values,
                spread_from,
            },
        );
    }
    if dynamic_scope {
        for call in result.values_mut() {
            call.values.fill(ArgumentValue::Unknown);
            call.spread_from = Some(0);
        }
    }
    result
}

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
            Expression::StringLiteral(value) if crate::ast::js_literal(value).len() <= 4096 => {
                ArgumentValue::Scalar(Value::String(crate::ast::js_literal(value).into_owned()))
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
                .filter(|part| part.value.cooked.is_some())
                .map(crate::ast::template_quasi_text)
                .filter(|text| text.len() <= 4096)
                .map_or(ArgumentValue::Unknown, |text| {
                    ArgumentValue::Scalar(Value::String(text.into_owned()))
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
                    let Some(key) = crate::ast::property_key_name(&property.key) else {
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
