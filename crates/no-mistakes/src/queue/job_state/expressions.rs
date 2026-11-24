use super::{Collector, Origin, StateKind, Value};
use crate::queue::extract_helpers::static_string;
use crate::queue::extract_job_id::referenced_symbol;
use crate::queue::extract_model::JobIdQueue;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, ChainElement, Expression, ObjectPropertyKind,
};
use oxc_syntax::operator::UnaryOperator;

pub(super) fn method<'node, 'a>(
    call: &'node CallExpression<'a>,
) -> Option<(&'node Expression<'a>, &'a str)> {
    let member = call.callee.get_member_expr()?;
    Some((member.object(), member.static_property_name()?.as_str()?))
}

impl Collector<'_, '_> {
    pub(super) fn value(&self, expression: &Expression<'_>) -> Option<Value> {
        match expression.get_inner_expression() {
            Expression::Identifier(id) => {
                let symbol = referenced_symbol(self.queue, id)?;
                self.flow
                    .values
                    .get(&symbol)
                    .cloned()
                    .filter(|value| match value {
                        Value::Item(origin) | Value::ItemData(origin) => {
                            self.flow.loops.contains(&origin.at)
                        }
                        _ => true,
                    })
            }
            Expression::AwaitExpression(awaited) => self.awaited(&awaited.argument),
            Expression::ObjectExpression(object) => object.properties.iter().find_map(|property| {
                let expression = match property {
                    ObjectPropertyKind::ObjectProperty(property) => &property.value,
                    ObjectPropertyKind::SpreadProperty(spread) => &spread.argument,
                };
                self.item_value(expression)
            }),
            Expression::ArrayExpression(array) => array.elements.iter().find_map(|element| {
                let expression = match element {
                    ArrayExpressionElement::SpreadElement(spread) => &spread.argument,
                    _ => element.as_expression()?,
                };
                self.item_value(expression)
            }),
            expression => {
                if let Some(member) = expression.get_member_expr() {
                    self.item_value(member.object())
                } else {
                    let (receiver, name) = method(as_call(expression)?)?;
                    if name != "map" {
                        return None;
                    }
                    match self.value(receiver)? {
                        Value::Listing(origin) | Value::MappedListing(origin) => {
                            Some(Value::MappedListing(origin))
                        }
                        _ => None,
                    }
                }
            }
        }
    }

    fn item_value(&self, expression: &Expression<'_>) -> Option<Value> {
        match self.value(expression)? {
            Value::Item(origin) | Value::ItemData(origin) => Some(Value::ItemData(origin)),
            _ => None,
        }
    }

    fn awaited(&self, expression: &Expression<'_>) -> Option<Value> {
        let Some(call) = as_call(expression) else {
            return self.value(expression);
        };
        let (receiver, name) = method(call)?;
        match name {
            "getJob" => self.origin(receiver, call).map(Value::Job),
            "getCompleted" | "getFailed" => self.origin(receiver, call).map(Value::Listing),
            "getJobs" if terminal_listing(call) => self.origin(receiver, call).map(Value::Listing),
            "getState" | "isFailed" | "isCompleted" => {
                let Value::Job(origin) = self.value(receiver)? else {
                    return None;
                };
                let kind = if name == "getState" {
                    StateKind::Name
                } else {
                    StateKind::Predicate
                };
                Some(Value::State(origin, kind))
            }
            _ => self.value(expression),
        }
    }

    pub(super) fn guard(&self, expression: &Expression<'_>) -> Option<Origin> {
        match expression.get_inner_expression() {
            Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
                self.guard(&unary.argument)
            }
            Expression::BinaryExpression(binary) if binary.operator.is_equality() => self
                .state_comparison(&binary.left, &binary.right)
                .or_else(|| self.state_comparison(&binary.right, &binary.left)),
            _ => match self.value(expression)? {
                Value::State(origin, StateKind::Predicate) => Some(origin),
                _ => None,
            },
        }
    }

    fn state_comparison(
        &self,
        state: &Expression<'_>,
        compared: &Expression<'_>,
    ) -> Option<Origin> {
        let Value::State(origin, kind) = self.value(state)? else {
            return None;
        };
        let literal = compared.get_inner_expression();
        let matches_kind = match kind {
            StateKind::Name => static_string(literal).is_some(),
            StateKind::Predicate => matches!(literal, Expression::BooleanLiteral(_)),
        };
        matches_kind.then_some(origin)
    }

    pub(super) fn queue_binding(&self, expression: &Expression<'_>) -> Option<JobIdQueue> {
        if let Expression::Identifier(id) = expression.get_inner_expression() {
            let symbol = referenced_symbol(self.queue, id)?;
            let scoping = self.queue.scoping?;
            let flags = scoping.symbol_flags(symbol);
            if flags.is_import() && !flags.is_type_import()
                || self.queue.job_id_queues.contains(&symbol)
            {
                return Some(JobIdQueue::Named(
                    self.queue.producer_binding(id.name.as_str(), id.span.start),
                ));
            }
        }
        let member = expression.get_member_expr()?;
        let Expression::Identifier(id) = member.object().get_inner_expression() else {
            return None;
        };
        let symbol = referenced_symbol(self.queue, id)?;
        let flags = self.queue.scoping?.symbol_flags(symbol);
        if !flags.is_import() || flags.is_type_import() {
            return None;
        }
        Some(JobIdQueue::Namespace {
            source: self.queue.namespace_imports.get(id.name.as_str())?.clone(),
            exported: member.static_property_name()?.as_str()?.to_string(),
        })
    }

    fn origin(&self, receiver: &Expression<'_>, call: &CallExpression<'_>) -> Option<Origin> {
        Some(Origin {
            binding: self.queue_binding(receiver)?,
            at: call.span.start,
        })
    }
}

fn as_call<'node, 'a>(expression: &'node Expression<'a>) -> Option<&'node CallExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::CallExpression(call) => Some(call),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => Some(call),
            _ => None,
        },
        _ => None,
    }
}

fn terminal_listing(call: &CallExpression<'_>) -> bool {
    let Some(Argument::ArrayExpression(states)) = call.arguments.first() else {
        return false;
    };
    states
        .elements
        .iter()
        .filter_map(ArrayExpressionElement::as_expression)
        .any(|value| {
            matches!(
                static_string(value.get_inner_expression()).as_deref(),
                Some("failed" | "completed")
            )
        })
}
