use super::*;

impl ImportCollector {
    pub(in super::super) fn invalidate_value_write(
        &mut self,
        scope: usize,
        name: &str,
        offset: u32,
    ) {
        self.invalidate_value_write_with_path(scope, name, offset, None, offset);
    }
    pub(in super::super) fn invalidate_value_write_with_path(
        &mut self,
        scope: usize,
        name: &str,
        offset: u32,
        receiver: Option<&[Option<String>]>,
        completed_at: u32,
    ) {
        let (base, member) = name
            .split_once('.')
            .map_or((name, false), |(base, _)| (base, true));
        let mut affected: FxHashMap<usize, FxHashSet<String>> = fx_map();
        affected.entry(scope).or_default().insert(base.to_string());
        if member && self.collect_value_flow {
            self.record_phase_scope(scope);
            self.prepare_value_references();
            let mut parts = receiver.map_or_else(
                || {
                    name.split('.')
                        .skip(1)
                        .map(|member| Some(member.to_string()))
                        .collect::<Vec<_>>()
                },
                |path| path.to_vec(),
            );
            // Exact assignments belong to the syntactic receiver/direct aliases.
            // Nested propagation keeps the existing default-origin prefix policy.
            let mut assigned = parts.pop().map(|member| (member, completed_at));
            let mut pending = vec![(((scope, base.to_string()), parts), false, true)];
            let mut active = fx_set();
            let mut visited = fx_set();
            while let Some(((key, path), exiting, remember)) = pending.pop() {
                if exiting {
                    active.remove(&key);
                    continue;
                }
                if remember {
                    self.remember_member_mutation(&key, &path, assigned.take(), offset);
                }
                if !visited.insert((key.clone(), path.clone())) || !active.insert(key.clone()) {
                    continue;
                }
                pending.push(((key.clone(), Vec::new()), true, false));
                affected.entry(key.0).or_default().insert(key.1.clone());
                for target in self
                    .value_reference_links
                    .as_ref()
                    .and_then(|links| links.get(&key.0))
                    .and_then(|names| names.get(key.1.as_str()))
                    .into_iter()
                    .flatten()
                {
                    pending.push(((target.clone(), path.clone()), false, false));
                }
                for index in self
                    .value_binding_indexes
                    .get(&key.0)
                    .and_then(|names| names.get(key.1.as_str()))
                    .into_iter()
                    .flatten()
                {
                    // Direct aliases are handled by the component links at finalization.
                    if matches!(&self.value_flow.bindings[*index].value,
                        ValueExpression::Named { name, .. } if !name.contains('.'))
                    {
                        continue;
                    }
                    let mut references = Vec::new();
                    nested::receiver_references(
                        &self.value_flow.bindings[*index].value,
                        &path,
                        &mut references,
                    );
                    pending.extend(
                        references
                            .into_iter()
                            .map(|reference| (reference, false, true)),
                    );
                }
            }
        }
        let context = self.current_function_id();
        for (scope, names) in &affected {
            for name in names {
                for index in self
                    .value_binding_indexes
                    .get(scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    self.value_flow.bindings[*index]
                        .mutation_contexts
                        .insert(context);
                    self.value_flow.bindings[*index]
                        .invalidated_at
                        .get_or_insert(offset);
                }
                for index in self
                    .value_parameter_indexes
                    .get(scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    self.value_flow.parameters[*index]
                        .mutation_contexts
                        .insert(context);
                    self.value_flow.parameters[*index]
                        .invalidated_at
                        .get_or_insert(offset);
                }
            }
        }
        let mut invalidated = Vec::new();
        for binding in &mut self.callable_aliases {
            let alias = &mut binding.alias;
            let head = alias
                .local
                .split_once('.')
                .map_or(alias.local.as_str(), |(head, _)| head);
            if member
                && affected
                    .get(&alias.binding_scope)
                    .is_some_and(|names| names.contains(head))
            {
                alias.invalidated_at.get_or_insert(offset);
                invalidated.push((alias.binding_scope, alias.local.clone()));
            }
        }
        for (scope, local) in invalidated {
            self.insert_reassigned_callable_at(scope, local);
        }
    }
}
impl ImportCollector {
    fn prepare_value_references(&mut self) {
        self.prepare_parameter_origins();
        if self.value_reference_links.is_some() {
            return;
        }
        let mut links = fx_map();
        for binding in &self.value_flow.bindings {
            insert_reference(&mut links, binding.scope, &binding.name, &binding.value);
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_link_constructions += 1;
            }
        }
        self.value_reference_links = Some(links);
    }
    pub(super) fn record_value_reference(
        &mut self,
        scope: usize,
        name: &str,
        value: &ValueExpression,
    ) {
        if let Some(links) = &mut self.value_reference_links {
            insert_reference(links, scope, name, value);
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_link_constructions += 1;
            }
        }
    }
}
fn insert_reference(links: &mut ReferenceLinks, scope: usize, name: &str, value: &ValueExpression) {
    if let Some((target_scope, target_name)) = referenced_value_root(value) {
        links
            .entry(scope)
            .or_default()
            .entry(name.to_string())
            .or_default()
            .push((target_scope, target_name.clone()));
        links
            .entry(target_scope)
            .or_default()
            .entry(target_name)
            .or_default()
            .push((scope, name.to_string()));
    }
}

fn referenced_value_root(value: &ValueExpression) -> Option<(usize, String)> {
    match value {
        ValueExpression::Named { name, scope, .. } if !name.contains('.') => Some((
            scope.unwrap_or(0),
            name.split_once('.')
                .map_or(name.as_str(), |(head, _)| head)
                .to_string(),
        )),

        _ => None,
    }
}
