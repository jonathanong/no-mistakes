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
            if !self.mutable_module_bindings.contains(&name) {
                continue;
            }
            let Some(scope) = self.callee_binding_scope(&name) else {
                continue;
            };
            if scope != 0 || self.imported_bindings.contains(&name) {
                continue;
            }
            self.value_flow
                .callable_assignments
                .push(extract_value_flow::CallableAssignment {
                    scope,
                    name,
                    completed_at,
                    callable,
                    owner: self.current_function_id(),
                    region_start: self.replacement_region.map(|region| region.start),
                    guaranteed: false,
                });
        }
    }
}
