use super::*;

impl ImportCollector {
    fn record_value_binding(&mut self, declarator: &VariableDeclarator<'_>, immutable: bool) {
        if !self.collect_value_flow {
            return;
        }
        if let Some(init) = &declarator.init {
            let value = if self
                .unreachable_value_binding_after
                .is_some_and(|cutoff| declarator.span.start >= cutoff)
            {
                ValueExpression::Unknown
            } else {
                self.value_expression(init)
            };
            if !immutable {
                for name in binding_names(&declarator.id) {
                    if let Some(scope) = self.callee_binding_scope(&name) {
                        self.invalidate_value_write(scope, &name, declarator.span.start);
                    }
                }
            }
            let factory_result = matches!(&value, ValueExpression::Call(_, _))
                || matches!(&value, ValueExpression::Awaited(inner) if matches!(inner.as_ref(), ValueExpression::Call(_, _)));
            if !immutable && !factory_result {
                return;
            }
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
                self.value_binding_indexes
                    .entry(scope)
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .push(self.value_flow.bindings.len());
                self.record_value_reference(scope, &name, &value);
                self.value_flow.bindings.push(ValueBinding {
                    owner: self.current_function_id(),
                    name,
                    scope,
                    declared_at,
                    invalidated_at: None,
                    mutation_contexts: fx_set(),
                    value,
                });
            }
            BindingPattern::ObjectPattern(object) => {
                let shared = std::sync::Arc::new(value);
                for property in &object.properties {
                    if !property.computed {
                        if let Some(key) =
                            crate::codebase::ts_source::static_property_key_name(&property.key)
                        {
                            self.record_value_pattern(
                                &property.value,
                                ValueExpression::Member(
                                    std::sync::Arc::clone(&shared),
                                    key.to_string(),
                                ),
                                declared_at,
                            );
                        }
                    }
                }
            }
            BindingPattern::AssignmentPattern(assignment) => self.record_value_pattern(
                &assignment.left,
                ValueExpression::Default(
                    std::sync::Arc::new(value),
                    std::sync::Arc::new(self.value_expression(&assignment.right)),
                ),
                declared_at,
            ),
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
            let first = self.value_flow.parameters.len();
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
            self.record_parameter_pattern(
                pattern,
                (owner, index),
                &default,
                self.value_flow.parameters.len(),
                Vec::new(),
            );
            if matches!(pattern, BindingPattern::BindingIdentifier(_)) {
                let reference = self.parameter_type_reference(param);
                for parameter in &mut self.value_flow.parameters[first..] {
                    parameter.type_reference.clone_from(&reference);
                }
            }
        }
    }
    pub(in super::super) fn record_value_call(&mut self, call: &CallExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        self.record_replacement_call(call);
        let (callee, arguments, spread_at, bound) = self.adapted_value_call(call);
        self.value_flow.calls.push(ValueCall {
            context: self.current_function_id(),
            offset: call.span.start,
            end: call.span.end,
            source_callee: recorded_call_callee(self, call)
                .unwrap_or_else(|| "<unknown>".to_string()),
            callee,
            arguments: std::sync::Arc::new(arguments),
            bound,
            spread_at,
            repeated: self.repeat_depth > self.call_repeat_base,
            once: self.current_function_id() == self.module_evaluation_owner
                && self.repeat_depth == 0
                && collect_helpers::direct_arrow(&call.callee),
        });
    }
}

impl ImportCollector {
    pub(in super::super) fn visit_value_return(
        &mut self,
        statement: &oxc_ast::ast::ReturnStatement<'_>,
    ) {
        self.invalidate_replacement_execution();
        if self.collect_value_flow {
            if let Some(owner) = self.current_function_id() {
                let value = statement
                    .argument
                    .as_ref()
                    .map_or(ValueExpression::Unknown, |argument| {
                        self.value_expression(argument)
                    });
                self.value_flow.returns.push((owner, value));
            }
        }
        walk::walk_return_statement(self, statement);
    }
}

impl ImportCollector {
    pub(in super::super) fn record_value_declaration(
        &mut self,
        declaration: &VariableDeclaration<'_>,
    ) {
        let immutable = matches!(
            declaration.kind,
            VariableDeclarationKind::Const
                | VariableDeclarationKind::Using
                | VariableDeclarationKind::AwaitUsing
        );
        for declarator in &declaration.declarations {
            self.record_value_binding(declarator, immutable);
        }
    }
}
