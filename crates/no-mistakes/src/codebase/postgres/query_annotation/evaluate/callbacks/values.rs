use super::*;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn live_callbacks(&self, id: u64) -> Vec<Value> {
        self.argument_slots(id)
            .filter(|(index, _)| !self.definite_deleted_argument_slots.contains(&(id, *index)))
            .map(|(_, value)| value.clone())
            .collect()
    }

    pub(super) fn callback_values(
        &mut self,
        values: &[Value],
        depth: u8,
        visited: &mut CallbackState,
    ) {
        self.retain_temporary_values(values);
        for value in values {
            match value {
                Value::Aggregate(values)
                | Value::References(values)
                | Value::Object(values)
                | Value::Record(values, _)
                | Value::Joined(values)
                | Value::Possible(values) => self.callback_values(values, depth, visited),
                Value::Promise(value) | Value::Evaluated(value, _) => {
                    self.callback_values(std::slice::from_ref(value.as_ref()), depth, visited);
                }
                Value::Arguments(id) | Value::Array(id)
                    if visited
                        .objects
                        .get(id)
                        .is_none_or(|(budget, _)| *budget < depth) =>
                {
                    let live = self.live_callbacks(*id);
                    visited.objects.insert(*id, (depth, live.clone()));
                    self.callback_values(&live, depth, visited);
                }
                Value::Function(function, path, captured) => {
                    let taints = self.invalidated_builders.len();
                    self.invalidate_captured(*captured, function);
                    if self.invalidated_builders.len() != taints {
                        visited.memo.clear();
                    }
                    for next_depth in depth.checked_sub(1).into_iter() {
                        let key = (
                            self.module_ids().id(path),
                            function.start,
                            *captured,
                            function.params.clone(),
                        );
                        if self
                            .active_callback_executions
                            .as_ref()
                            .unwrap()
                            .contains(&key)
                        {
                            continue;
                        }
                        if self
                            .active_callback_functions
                            .as_mut()
                            .expect("active opaque consumer")
                            .insert(key.clone())
                        {
                            self.active_callback_executions
                                .as_mut()
                                .unwrap()
                                .insert(key.clone());
                            let snapshot = self.memoized_callback_snapshot(
                                path,
                                *captured,
                                function,
                                depth,
                                &mut visited.memo,
                            );
                            visited
                                .functions
                                .insert(key.clone(), (depth, value.clone(), snapshot));
                            let mut locals = scopes::locals(&self.scopes[*captured], function);
                            if let Some(name) = &function.self_name {
                                locals.insert(name.clone(), Value::Unknown);
                            }
                            scopes::arguments(&mut locals, function, Value::Unknown);
                            for name in &function.params {
                                locals.insert(name.clone(), Value::Unknown);
                            }
                            // Opaque consumers do not supply a reference receiver.
                            scopes::bind_this(&mut locals, function, None);
                            let mut scope = self.environment(locals);
                            self.register_mappings(scope, *captured, function, None);
                            self.register_captured_bindings(scope, *captured, function);
                            let mutating = self.callback_may_mutate(function, path, scope);
                            if mutating {
                                visited.memo.clear();
                            }
                            let returned =
                                self.steps(&function.body, path, &mut scope, next_depth, false);
                            self.retain_temporary_roots(&returned);
                            if mutating {
                                visited.memo.clear();
                            }
                            self.callback_values(
                                std::slice::from_ref(&returned),
                                next_depth,
                                visited,
                            );
                            self.active_callback_executions
                                .as_mut()
                                .unwrap()
                                .remove(&key);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
