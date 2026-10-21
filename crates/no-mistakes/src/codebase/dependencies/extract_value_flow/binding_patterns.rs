use super::*;

impl ImportCollector {
    pub(super) fn record_value_pattern(
        &mut self,
        pattern: &BindingPattern<'_>,
        value: ValueExpression,
        declared_at: u32,
        immutable: bool,
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
                    immutable,
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
                                immutable,
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
                immutable,
            ),
            BindingPattern::ArrayPattern(array) => {
                if immutable {
                    // Retain iterable provenance without introducing runtime
                    // receiver bindings or assuming an iterable's element value.
                    for element in array.elements.iter().flatten() {
                        for name in binding_names(element) {
                            let scope = self
                                .callee_binding_scope(&name)
                                .unwrap_or(self.current_lexical_scope_id());
                            self.value_flow.iteration_bindings.push(ValueBinding {
                                immutable,
                                owner: self.current_function_id(),
                                name,
                                scope,
                                declared_at,
                                repeated: self.repeat_depth > self.call_repeat_base,
                                invalidated_at: None,
                                mutation_contexts: fx_set(),
                                value: value.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}
