use super::*;
use oxc_syntax::operator::BinaryOperator;

impl ImportCollector {
    pub(super) fn note_sql_placeholder_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
        declared_at: u32,
    ) {
        let BindingPattern::BindingIdentifier(identifier) = pattern else {
            return;
        };
        let name = identifier.name.to_string();
        let scope = self.callee_binding_scope(&name).unwrap_or(0);
        match crate::codebase::ts_source::unwrap_ts_wrappers(init) {
            Expression::ArrayExpression(array)
                if array.elements.iter().all(|element| {
                    element
                        .as_expression()
                        .is_some_and(|value| self.placeholder_text(value))
                }) =>
            {
                self.value_flow
                    .sql_placeholder_lists
                    .push((scope, name, declared_at));
            }
            Expression::CallExpression(call) => {
                let Expression::StaticMemberExpression(member) = &call.callee else {
                    return;
                };
                if member.property.name != "join"
                    || call.arguments.len() != 1
                    || call.arguments[0]
                        .as_expression()
                        .and_then(closed_identifiers::literal_string)
                        .is_none()
                {
                    return;
                }
                let Expression::Identifier(list) = &member.object else {
                    return;
                };
                let list_scope = self.callee_binding_scope(list.name.as_str()).unwrap_or(0);
                if self
                    .value_flow
                    .sql_placeholder_lists
                    .iter()
                    .any(|(scope, name, _)| *scope == list_scope && name == list.name.as_str())
                {
                    self.value_flow
                        .bounded_bindings
                        .push(closed_identifiers::BoundedBinding {
                            name,
                            scope,
                            declared_at,
                            source: Some((list.name.to_string(), list_scope)),
                            members: Vec::new(),
                            requires_guard: false,
                        });
                }
            }
            _ => {}
        }
    }

    fn placeholder_text(&self, value: &Expression<'_>) -> bool {
        let value = crate::codebase::ts_source::unwrap_ts_wrappers(value);
        if closed_identifiers::literal_string(value).is_some() {
            return true;
        }
        match value {
            Expression::TemplateLiteral(template) => {
                template.expressions.iter().enumerate().all(|(i, hole)| {
                    crate::ast::template_quasi_text(&template.quasis[i]).ends_with('$')
                        && self.integer_placeholder(hole)
                })
            }
            Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
                closed_identifiers::literal_string(&binary.left)
                    .is_some_and(|text| text.ends_with('$'))
                    && self.integer_placeholder(&binary.right)
            }
            _ => false,
        }
    }

    fn integer_placeholder(&self, value: &Expression<'_>) -> bool {
        match crate::codebase::ts_source::unwrap_ts_wrappers(value) {
            Expression::NumericLiteral(number) => {
                number.value.is_finite() && number.value.fract() == 0.0
            }
            Expression::Identifier(identifier) => {
                let scope = self.callee_binding_scope(identifier.name.as_str());
                self.value_flow.parameters.iter().any(|parameter| {
                    Some(parameter.scope) == scope
                        && parameter.name == identifier.name.as_str()
                        && parameter.type_reference == Some(ValueType::Number)
                })
            }
            Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
                self.integer_placeholder(&binary.left) && self.integer_placeholder(&binary.right)
            }
            _ => false,
        }
    }
}
