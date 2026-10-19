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
                            self.value_expression(expression)
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
