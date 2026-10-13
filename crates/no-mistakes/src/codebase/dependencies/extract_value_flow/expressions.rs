use super::*;

impl ImportCollector {
    pub(in super::super) fn value_expression(
        &self,
        expression: &Expression<'_>,
    ) -> ValueExpression {
        match expression {
            Expression::AwaitExpression(awaited) => self.value_expression(&awaited.argument),
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
            Expression::CallExpression(call) => {
                ValueExpression::Call(call.span.start, call.span.end)
            }
            Expression::ArrowFunctionExpression(arrow) => {
                ValueExpression::Callable(CallableId(arrow.span.start))
            }
            Expression::FunctionExpression(function) => {
                ValueExpression::Callable(CallableId(function.span.start))
            }
            Expression::ObjectExpression(object) => {
                let mut fields = Vec::new();
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return ValueExpression::Unknown;
                    };
                    if property.computed || property.method || property.kind != PropertyKind::Init {
                        return ValueExpression::Unknown;
                    }
                    let Some(key) =
                        crate::codebase::ts_source::static_property_key_name(&property.key)
                    else {
                        return ValueExpression::Unknown;
                    };
                    fields.push((key.to_string(), self.value_expression(&property.value)));
                }
                ValueExpression::Object(fields)
            }
            _ => simple_callee_name(expression).map_or(ValueExpression::Unknown, |name| {
                ValueExpression::Named {
                    scope: self.callee_binding_scope(&name),
                    name,
                    offset: expression.span().start,
                }
            }),
        }
    }
}
