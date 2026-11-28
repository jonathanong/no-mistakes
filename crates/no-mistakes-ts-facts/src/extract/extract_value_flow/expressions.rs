use super::*;
use oxc_ast::ast::ConditionalExpression;

#[path = "expressions/string_assertions.rs"]
mod string_assertions;

impl ImportCollector {
    pub(in super::super) fn value_expression(
        &self,
        expression: &Expression<'_>,
    ) -> ValueExpression {
        match unwrap_value_wrappers(expression) {
            Expression::Identifier(identifier)
                if identifier.name == "undefined"
                    && self.callee_binding_scope("undefined").is_none()
                    && !self.imported_bindings.contains("undefined")
                    && !self.predeclared_imported_bindings.contains("undefined") =>
            {
                ValueExpression::Undefined
            }
            Expression::UnaryExpression(unary)
                if unary.operator == oxc_syntax::operator::UnaryOperator::Void =>
            {
                ValueExpression::Undefined
            }
            Expression::TaggedTemplateExpression(tagged) => {
                ValueExpression::Call(templates::tag_offset(tagged), tagged.span.end)
            }
            Expression::TemplateLiteral(template) if !template.expressions.is_empty() => {
                ValueExpression::Derived(
                    template
                        .expressions
                        .iter()
                        .map(|part| self.value_expression(part))
                        .collect::<Vec<_>>()
                        .into(),
                )
            }
            // A comma expression evaluates to its final operand only.
            Expression::SequenceExpression(sequence) => sequence
                .expressions
                .last()
                .map_or(ValueExpression::Unknown, |last| self.value_expression(last)),
            Expression::ConditionalExpression(value) => self.conditional_value_expression(value),
            Expression::BinaryExpression(value)
                if value.operator == oxc_syntax::operator::BinaryOperator::Addition =>
            {
                ValueExpression::Derived(
                    vec![
                        self.value_expression(&value.left),
                        self.value_expression(&value.right),
                    ]
                    .into(),
                )
            }
            Expression::StaticMemberExpression(member) => ValueExpression::Member(
                std::sync::Arc::new(self.value_expression(&member.object)),
                member.property.name.to_string(),
            ),
            Expression::ComputedMemberExpression(member) => write_targets::static_key(
                &member.expression,
            )
            .map_or(ValueExpression::Unknown, |key| {
                ValueExpression::Member(
                    std::sync::Arc::new(self.value_expression(&member.object)),
                    key,
                )
            }),
            Expression::Super(_) => self
                .current_function_id()
                .map_or(ValueExpression::Unknown, ValueExpression::Super),
            Expression::AwaitExpression(awaited) => ValueExpression::Awaited(std::sync::Arc::new(
                self.value_expression(&awaited.argument),
            )),
            Expression::CallExpression(call) => self
                .object_assign_result(call)
                .unwrap_or(ValueExpression::Call(call.span.start, call.span.end)),
            Expression::ArrowFunctionExpression(arrow) => {
                ValueExpression::Callable(CallableId(arrow.span.start))
            }
            Expression::FunctionExpression(function) => {
                ValueExpression::Callable(CallableId(function.span.start))
            }
            Expression::ArrayExpression(array) => self.array_value_expression(array),
            Expression::LogicalExpression(logical)
                if logical.operator == oxc_syntax::operator::LogicalOperator::Coalesce =>
            {
                ValueExpression::Nullish(
                    std::sync::Arc::new(self.value_expression(&logical.left)),
                    std::sync::Arc::new(self.value_expression(&logical.right)),
                )
            }
            Expression::NullLiteral(_) => ValueExpression::Null,
            Expression::ObjectExpression(object) => self.object_value(object),
            other => simple_callee_name(other).map_or(ValueExpression::Unknown, |name| {
                ValueExpression::Named {
                    scope: self.callee_binding_scope(&name),
                    name,
                    offset: other.span().start,
                    context: self.current_function_id(),
                }
            }),
        }
    }

    fn array_value_expression(&self, array: &oxc_ast::ast::ArrayExpression<'_>) -> ValueExpression {
        let mut fields = fx_map();
        for (index, element) in array.elements.iter().enumerate() {
            if matches!(
                element,
                oxc_ast::ast::ArrayExpressionElement::SpreadElement(_)
            ) {
                return ValueExpression::Unknown;
            }
            let value = element
                .as_expression()
                .map_or(ValueExpression::Undefined, |expression| {
                    super::receiver_escapes::array_element_value(self, expression)
                });
            fields.insert(index.to_string(), value);
        }
        ValueExpression::Array(std::sync::Arc::new(fields))
    }

    fn conditional_value_expression(&self, value: &ConditionalExpression<'_>) -> ValueExpression {
        let consequent = self.value_expression(&value.consequent);
        let alternate = self.value_expression(&value.alternate);
        if literal_conditional_arm(&value.consequent, &consequent)
            && literal_conditional_arm(&value.alternate, &alternate)
        {
            // Preserve the existing finite-return proof without treating
            // opaque templates or other Derived values as literals.
            ValueExpression::Unknown
        } else {
            ValueExpression::Derived(vec![consequent, alternate].into())
        }
    }
}

fn unwrap_value_wrappers<'a>(mut expr: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        expr = match expr {
            Expression::ParenthesizedExpression(parenthesized) => &parenthesized.expression,
            Expression::TSAsExpression(assertion) => &assertion.expression,
            Expression::TSSatisfiesExpression(assertion) => &assertion.expression,
            Expression::TSNonNullExpression(assertion) => &assertion.expression,
            Expression::TSInstantiationExpression(instantiation) => &instantiation.expression,
            _ => return expr,
        };
    }
}

fn literal_conditional_arm(expression: &Expression<'_>, value: &ValueExpression) -> bool {
    let expression = crate::syntax_helpers::unwrap_ts_wrappers(expression);
    expression.is_literal()
        || matches!(expression, Expression::TemplateLiteral(template) if template.expressions.is_empty())
        || (matches!(expression, Expression::ConditionalExpression(_))
            && matches!(value, ValueExpression::Unknown))
}
