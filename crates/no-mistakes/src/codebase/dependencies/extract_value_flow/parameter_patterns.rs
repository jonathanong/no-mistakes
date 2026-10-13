use super::*;

impl ImportCollector {
    pub(super) fn record_parameter_pattern(
        &mut self,
        pattern: &BindingPattern<'_>,
        slot: (CallableId, usize),
        default: &ValueExpression,
        default_order: usize,
        projection: Vec<ValueProjection>,
    ) {
        let (owner, index) = slot;
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => {
                let name = identifier.name.to_string();
                let scope = self.current_lexical_scope_id();
                self.value_parameter_indexes
                    .entry(scope)
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .push(self.value_flow.parameters.len());
                self.value_flow.parameters.push(ValueParameter {
                    owner,
                    index,
                    name,
                    scope,
                    default: default.clone(),
                    projection,
                    initialization_order: self.value_flow.parameters.len(),
                    default_order,
                    invalidated_at: None,
                    mutation_contexts: fx_set(),
                });
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    if !property.computed {
                        if let Some(key) =
                            crate::codebase::ts_source::static_property_key_name(&property.key)
                        {
                            let mut steps = projection.clone();
                            steps.push(ValueProjection::Member(key.to_string()));
                            self.record_parameter_pattern(
                                &property.value,
                                slot,
                                default,
                                default_order,
                                steps,
                            );
                        }
                    }
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for (position, element) in array.elements.iter().enumerate() {
                    if let Some(element) = element {
                        let mut steps = projection.clone();
                        steps.push(ValueProjection::Element(position.to_string()));
                        self.record_parameter_pattern(element, slot, default, default_order, steps);
                    }
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                let mut steps = projection;
                steps.push(ValueProjection::Default(
                    self.value_expression(&assignment.right),
                    self.value_flow.parameters.len(),
                ));
                self.record_parameter_pattern(
                    &assignment.left,
                    slot,
                    default,
                    default_order,
                    steps,
                );
            }
        }
        // Computed keys and rest bindings remain unresolved; they cannot manufacture provenance.
    }
}
