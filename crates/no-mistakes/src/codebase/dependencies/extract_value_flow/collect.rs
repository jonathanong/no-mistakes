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
            Expression::CallExpression(call) => ValueExpression::Call(call.span.start),
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
    pub(in super::super) fn record_value_binding(&mut self, declarator: &VariableDeclarator<'_>) {
        if !self.collect_value_flow {
            return;
        }
        if let Some(init) = &declarator.init {
            let value = self.value_expression(init);
            self.record_value_pattern(&declarator.id, value, declarator.span.start);
        }
    }
    fn record_value_pattern(
        &mut self,
        pattern: &BindingPattern<'_>,
        value: ValueExpression,
        declared_at: u32,
    ) {
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => {
                let name = identifier.name.to_string();
                let scope = self
                    .callee_binding_scope(&name)
                    .unwrap_or(self.current_lexical_scope_id());
                self.value_flow.bindings.push(ValueBinding {
                    name,
                    scope,
                    declared_at,
                    invalidated_at: None,
                    value,
                });
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    if !property.computed {
                        if let Some(key) =
                            crate::codebase::ts_source::static_property_key_name(&property.key)
                        {
                            self.record_value_pattern(
                                &property.value,
                                ValueExpression::Member(Box::new(value.clone()), key.to_string()),
                                declared_at,
                            );
                        }
                    }
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.record_value_pattern(&assignment.left, value, declared_at)
            }
            BindingPattern::ArrayPattern(_) => {}
        }
    }
    pub(in super::super) fn record_value_parameters(&mut self, params: &FormalParameters<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let Some(owner) = self.current_function_id() else {
            return;
        };
        for (index, param) in params.items.iter().enumerate() {
            let (pattern, default) = match &param.pattern {
                BindingPattern::AssignmentPattern(assignment) => {
                    (&assignment.left, self.value_expression(&assignment.right))
                }
                pattern => (
                    pattern,
                    param
                        .initializer
                        .as_ref()
                        .map_or(ValueExpression::Unknown, |expression| {
                            self.value_expression(expression)
                        }),
                ),
            };
            if let Some(name) = binding_identifier_name(pattern) {
                self.value_flow.parameters.push(ValueParameter {
                    owner,
                    index,
                    name: name.to_string(),
                    scope: self.current_lexical_scope_id(),
                    default,
                    invalidated_at: None,
                });
            }
        }
    }
    pub(in super::super) fn record_value_call(&mut self, call: &CallExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let callee = self.value_expression(&call.callee);
        let arguments = call
            .arguments
            .iter()
            .map(|argument| {
                argument
                    .as_expression()
                    .map_or(ValueExpression::Unknown, |expression| {
                        self.value_expression(expression)
                    })
            })
            .collect();
        self.value_flow.calls.push(ValueCall {
            offset: call.span.start,
            callee,
            arguments,
            spread_at: call
                .arguments
                .iter()
                .position(|argument| argument.as_expression().is_none()),
        });
    }
}

impl ImportCollector {
    pub(in super::super) fn visit_value_return(
        &mut self,
        statement: &oxc_ast::ast::ReturnStatement<'_>,
    ) {
        if self.collect_value_flow {
            if let (Some(owner), Some(argument)) =
                (self.current_function_id(), statement.argument.as_ref())
            {
                let value = self.value_expression(argument);
                self.value_flow.returns.push((owner, value));
            }
        }
        walk::walk_return_statement(self, statement);
    }
}
