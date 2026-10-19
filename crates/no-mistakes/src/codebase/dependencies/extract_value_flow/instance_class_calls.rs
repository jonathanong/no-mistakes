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
        self.class_method_calls
            .push((call.span.start, method, arguments));
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
        let mut position = 0usize;
        let mut open_from: Option<usize> = None;
        let mut open_values = Vec::new();
        for (raw_index, argument) in call.arguments.iter().enumerate() {
            match argument {
                Argument::SpreadElement(spread) => {
                    let value = self.value_expression(&spread.argument);
                    match super::parameter_call_origins::array_entries(self, &value) {
                        Some(entries) if open_from.is_none() => {
                            let mut next = position;
                            for (slot, value) in entries {
                                let at = position + slot;
                                extras.entry(at).or_default().push(value);
                                next = next.max(at + 1);
                            }
                            position = next;
                        }
                        Some(entries) => {
                            open_values.extend(entries.into_iter().map(|(_, value)| value));
                        }
                        None => {
                            open_from.get_or_insert(position);
                        }
                    }
                }
                other => {
                    let values = other.as_expression().map_or_else(Vec::new, |expression| {
                        candidate_argument_values(self, expression)
                    });
                    if open_from.is_some() {
                        open_values.extend(values);
                    } else if values.len() != 1 || position != raw_index {
                        if !values.is_empty() {
                            extras.entry(position).or_default().extend(values);
                        }
                        position += 1;
                    } else {
                        position += 1;
                    }
                }
            }
        }
        if !extras.is_empty() {
            self.escaped_parameter_arguments
                .insert(call.span.start, extras);
        }
        if let Some(start) = open_from.filter(|_| !open_values.is_empty()) {
            self.escaped_open_arguments
                .insert(call.span.start, (start, open_values));
        }
    }
}

/// Both sides of a conditional or logical expression can be the runtime argument.
fn candidate_argument_values(
    collector: &ImportCollector,
    expression: &Expression<'_>,
) -> Vec<ValueExpression> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ConditionalExpression(conditional) => {
            let mut values = candidate_argument_values(collector, &conditional.consequent);
            values.extend(candidate_argument_values(collector, &conditional.alternate));
            values
        }
        Expression::LogicalExpression(logical) => {
            let mut values = candidate_argument_values(collector, &logical.left);
            values.extend(candidate_argument_values(collector, &logical.right));
            values
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .map(|expression| candidate_argument_values(collector, expression))
            .unwrap_or_default(),
        other => vec![collector.value_expression(other)],
    }
}
