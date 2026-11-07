impl ImportCollector {
    fn invalidate_bounded_name(&mut self, name: &str, container_only: bool) {
        if !self.collect_per_item_calls {
            return;
        }
        let name = name.split_once('.').map_or(name, |(root, _)| root);
        for (depth, scope) in self.lexical_scope_ids.iter().enumerate().rev() {
            if let Some(value) = self
                .bounded_domains
                .values
                .get_mut(scope)
                .and_then(|values| values.get_mut(name))
            {
                if !container_only || !matches!(value, BoundedValue::Number(_)) {
                    *value = BoundedValue::Unknown;
                }
                return;
            }
            if self.local_stack[depth].contains(name) {
                return;
            }
        }
    }

    fn invalidate_bounded_expression(&mut self, expression: &Expression<'_>) {
        if !self.collect_per_item_calls {
            return;
        }
        #[derive(Default)]
        struct Names {
            names: FxHashSet<String>,
        }
        impl<'a> Visit<'a> for Names {
            fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
                self.names.insert(identifier.name.to_string());
            }
            fn visit_static_member_expression(&mut self, _: &StaticMemberExpression<'a>) {}
            fn visit_computed_member_expression(&mut self, _: &ComputedMemberExpression<'a>) {}
        }
        let mut names = Names::default();
        names.visit_expression(expression);
        for name in names.names {
            self.invalidate_bounded_name(&name, true);
        }
    }

    fn invalidate_bounded_call(&mut self, call: &CallExpression<'_>) {
        if !self.collect_per_item_calls {
            return;
        }
        let callee = simple_callee_name(&call.callee);
        if callee.as_deref() == Some("eval") {
            for scope in &self.lexical_scope_ids {
                if let Some(values) = self.bounded_domains.values.get_mut(scope) {
                    for value in values.values_mut() {
                        if !matches!(value, BoundedValue::Number(_)) {
                            *value = BoundedValue::Unknown;
                        }
                    }
                }
            }
        }
        let object_read = self.builtin_bounded_object()
            && matches!(
                callee.as_deref(),
                Some("Object.entries" | "Object.keys" | "Object.values")
            )
            && call.arguments.len() == 1;
        let iteration = callee
            .as_ref()
            .and_then(|name| name.rsplit_once('.'))
            .is_some_and(|(_, method)| {
                matches!(
                    method,
                    "map" | "flatMap" | "forEach" | "filter" | "some" | "every" | "reduce" | "find"
                )
            });
        if !iteration {
            let receiver = match &call.callee {
                Expression::StaticMemberExpression(member) => Some(&member.object),
                Expression::ComputedMemberExpression(member) => Some(&member.object),
                _ => None,
            };
            if let Some(receiver) = receiver {
                if let Some(name) = simple_callee_name(receiver) {
                    self.invalidate_bounded_name(&name, true);
                }
            }
        }
        if object_read {
            return;
        }
        for argument in &call.arguments {
            match argument {
                Argument::SpreadElement(spread) => {
                    self.invalidate_bounded_expression(&spread.argument)
                }
                argument => {
                    if let Some(expression) = argument.as_expression() {
                        self.invalidate_bounded_expression(expression);
                    }
                }
            }
        }
    }
}
