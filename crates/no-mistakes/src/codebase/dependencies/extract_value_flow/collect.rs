use super::*;
#[path = "for_of_value_binding.rs"]
mod for_of_value_binding;

impl ImportCollector {
    fn record_value_binding(&mut self, declarator: &VariableDeclarator<'_>, immutable: bool) {
        if !self.collect_value_flow {
            return;
        }
        self.note_instance_class(declarator);
        if let Some(init) = &declarator.init {
            self.record_callable_initializer(declarator, immutable, init);
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
                self.record_mutable_receiver_binding(&declarator.id, value, declarator.span.end);
                return;
            }
            self.record_value_pattern(&declarator.id, value, declarator.span.start);
        }
    }
    pub(super) fn record_value_pattern(
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
                    repeated: self.repeat_depth > self.call_repeat_base,
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
        self.note_class_method_call(call);
        self.note_escaped_arguments(call);
        let (callee, arguments, spread_at, bound) = self.adapted_value_call(call);
        self.record_object_assign_providers(&callee, &arguments, &call.arguments, call.span.end);
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

include!("callable_assignment_initializers.rs");

impl ImportCollector {
    fn note_instance_class(&mut self, declarator: &VariableDeclarator<'_>) {
        let Some(init) = &declarator.init else {
            return;
        };
        let Expression::NewExpression(new) = crate::codebase::ts_source::unwrap_ts_wrappers(init)
        else {
            return;
        };
        let Some(name) = simple_callee_name(&new.callee) else {
            return;
        };
        let Some(class_id) =
            self.callable_binding_at(self.callee_binding_scope(&name).unwrap_or(0), &name)
        else {
            return;
        };
        for binding in binding_names(&declarator.id) {
            let scope = self
                .callee_binding_scope(&binding)
                .unwrap_or(self.current_lexical_scope_id());
            self.instance_classes.insert((scope, binding), class_id);
        }
    }

    fn note_class_method_call(&mut self, call: &CallExpression<'_>) {
        let Some(method) = self.class_method_id(call) else {
            return;
        };
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
        self.class_method_calls.push((method, arguments));
    }

    fn class_method_id(&self, call: &CallExpression<'_>) -> Option<CallableId> {
        let Expression::StaticMemberExpression(member) =
            crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
        else {
            return None;
        };
        let class = match crate::codebase::ts_source::unwrap_ts_wrappers(&member.object) {
            Expression::NewExpression(new) => {
                let name = simple_callee_name(&new.callee)?;
                self.callable_binding_at(self.callee_binding_scope(&name).unwrap_or(0), &name)?
            }
            Expression::Identifier(identifier) => {
                let scope = self
                    .callee_binding_scope(identifier.name.as_str())
                    .unwrap_or(0);
                *self
                    .instance_classes
                    .get(&(scope, identifier.name.to_string()))?
            }
            _ => return None,
        };
        owner_member_id(
            &self.aggregate_callable_member_ids,
            class,
            member.property.name.as_str(),
        )
    }

    fn note_escaped_arguments(&mut self, call: &CallExpression<'_>) {
        let mut extras: FxHashMap<usize, Vec<ValueExpression>> = fx_map();
        for (index, argument) in call.arguments.iter().enumerate() {
            match argument {
                Argument::SpreadElement(spread) => {
                    if let ValueExpression::Array(items) = self.value_expression(&spread.argument) {
                        let mut entries: Vec<_> = items.iter().collect();
                        entries.sort_by(|left, right| left.0.cmp(right.0));
                        for (offset, (_, value)) in entries.into_iter().enumerate() {
                            extras
                                .entry(index + offset)
                                .or_default()
                                .push(value.clone());
                        }
                    }
                }
                other => {
                    if let Some(Expression::ConditionalExpression(conditional)) = other
                        .as_expression()
                        .map(crate::codebase::ts_source::unwrap_ts_wrappers)
                    {
                        let values = extras.entry(index).or_default();
                        values.push(self.value_expression(&conditional.consequent));
                        values.push(self.value_expression(&conditional.alternate));
                    }
                }
            }
        }
        if !extras.is_empty() {
            self.escaped_parameter_arguments
                .insert(call.span.start, extras);
        }
    }
}
