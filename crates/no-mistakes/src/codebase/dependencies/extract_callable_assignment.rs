impl ImportCollector {
    fn record_callable_assignment(
        &mut self,
        target: &AssignmentTarget<'_>,
        right: Option<&Expression<'_>>,
        plain: bool,
        completed_at: u32,
    ) {
        if !self.collect_value_flow {
            return;
        }
        // Member writes cannot assign a lexical module binding. Avoid building
        // owned receiver paths for callable-only staging; receiver facts are
        // still recorded by the assignment visitor independently.
        if self.callable_assignment_state.is_none()
            || matches!(
                target,
                AssignmentTarget::StaticMemberExpression(_)
                    | AssignmentTarget::ComputedMemberExpression(_)
            )
        {
            return;
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.callable_assignment_target_constructions += 1;
        }
        let literal = plain && matches!(target, AssignmentTarget::AssignmentTargetIdentifier(_));
        let callable = literal
            .then(|| {
                right.and_then(|right| {
                    match crate::codebase::ts_source::unwrap_ts_wrappers(right) {
                        Expression::FunctionExpression(function) => {
                            Some(CallableId(function.span.start))
                        }
                        Expression::ArrowFunctionExpression(arrow) => {
                            Some(CallableId(arrow.span.start))
                        }
                        _ => None,
                    }
                })
            })
            .flatten();
        for (name, _, _) in extract_value_flow::write_targets::targets(target) {
            if !self.is_mutable_module_binding(&name) {
                continue;
            }
            let Some(scope) = self.callee_binding_scope(&name) else {
                continue;
            };
            if scope != 0 || self.imported_bindings.contains(&name) {
                continue;
            }
            let owner = self.current_function_id();
            self.callable_assignment_state
                .as_mut()
                .expect("mutable module state")
                .assignments
                .push(extract_value_flow::CallableAssignment {
                    scope,
                    name,
                    completed_at,
                    callable,
                    owner,
                    region_start: self.replacement_region.map(|region| region.start),
                    guaranteed: false,
                });
        }
    }
}
