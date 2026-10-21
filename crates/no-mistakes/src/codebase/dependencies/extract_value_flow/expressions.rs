use super::*;

impl ImportCollector {
    pub(in super::super) fn value_expression(
        &self,
        expression: &Expression<'_>,
    ) -> ValueExpression {
        match expression {
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
            Expression::ConditionalExpression(value) => {
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
            Expression::ParenthesizedExpression(parenthesized) => {
                self.value_expression(&parenthesized.expression)
            }
            Expression::TSAsExpression(assertion) => self.value_expression(&assertion.expression),
            Expression::TSSatisfiesExpression(assertion) => {
                self.value_expression(&assertion.expression)
            }
            Expression::TSNonNullExpression(assertion) => {
                self.value_expression(&assertion.expression)
            }
            Expression::CallExpression(call) => self
                .object_assign_result(call)
                .unwrap_or(ValueExpression::Call(call.span.start, call.span.end)),
            Expression::ArrowFunctionExpression(arrow) => {
                ValueExpression::Callable(CallableId(arrow.span.start))
            }
            Expression::FunctionExpression(function) => {
                ValueExpression::Callable(CallableId(function.span.start))
            }
            Expression::ArrayExpression(array) => {
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
            _ => simple_callee_name(expression).map_or(ValueExpression::Unknown, |name| {
                ValueExpression::Named {
                    scope: self.callee_binding_scope(&name),
                    name,
                    offset: expression.span().start,
                    context: self.current_function_id(),
                }
            }),
        }
    }
}

fn literal_conditional_arm(expression: &Expression<'_>, value: &ValueExpression) -> bool {
    let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
    expression.is_literal()
        || matches!(expression, Expression::TemplateLiteral(template) if template.expressions.is_empty())
        || (matches!(expression, Expression::ConditionalExpression(_))
            && matches!(value, ValueExpression::Unknown))
}
