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
            self.instance_classes
                .entry(scope)
                .or_default()
                .insert(binding, class_id);
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
                self.instance_classes
                    .get(&scope)
                    .and_then(|classes| classes.get(identifier.name.as_str()))
                    .copied()?
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
