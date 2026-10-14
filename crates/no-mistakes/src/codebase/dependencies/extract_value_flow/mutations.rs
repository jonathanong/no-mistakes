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
            // A nested alias writes the same member on its original receiver.
            // Keep exact assignment metadata alongside every propagated path.
            let assigned_member = match receiver {
                Some(path) => path.last().and_then(Option::as_deref),
                None => name.rsplit_once('.').map(|(_, member)| member),
            };
            let mut assigned = parts.pop().map(|member| (member, completed_at));
            let has_assignment = assigned.is_some();
            let reads = vec![Some((offset, self.current_function_id())); parts.len()];
            let mut pending = vec![(
                (((scope, base.to_string()), parts), reads),
                false,
                true,
                false,
            )];
            let mut active = fx_set();
            let mut visited = fx_map();
            while let Some((((key, path), reads), exiting, remember, propagated)) = pending.pop() {
                if exiting {
                    active.remove(&key);
                    continue;
                }
                if remember {
                    let exact = assigned.take().or_else(|| {
                        has_assignment.then(|| (assigned_member.map(str::to_string), completed_at))
                    });
                    self.remember_member_mutation(&key, &path, exact, offset, propagated);
                }
                // A cycle must not mark an unpropagated weaker summary visited.
                if active.contains(&key) {
                    continue;
                }
                let Some(reads) = receiver_traversal::merge_reads(&mut visited, &key, &path, reads)
                else {
                    continue;
                };
                active.insert(key.clone());
                #[cfg(feature = "test-instrumentation")]
                {
                    self.value_flow.reference_path_visits += 1;
                }
                pending.push((((key.clone(), Vec::new()), Vec::new()), true, false, false));
                affected.entry(key.0).or_default().insert(key.1.clone());
                for target in self
                    .value_reference_links
                    .as_ref()
                    .and_then(|links| links.get(&key.0))
                    .and_then(|names| names.get(key.1.as_str()))
                    .into_iter()
                    .flatten()
                {
                    #[cfg(feature = "test-instrumentation")]
                    {
                        self.value_flow.reference_link_visits += 1;
                    }
                    pending.push((
                        ((target.clone(), path.clone()), reads.clone()),
                        false,
                        false,
                        propagated,
                    ));
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
                    if self.reference_path_may_be_replaced(&key, &path, &reads) {
                        continue;
                    }
                    let mut references = Vec::new();
                    nested::write_receiver_references(
                        &self.value_flow.bindings[*index].value,
                        &path,
                        &reads,
                        &mut references,
                    );
                    pending.extend(
                        references
                            .into_iter()
                            .map(|reference| (reference, false, true, true)),
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
    fn reference_path_may_be_replaced(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
    ) -> bool {
        if path.is_empty() {
            return false;
        }
        self.prior_member_mutations
            .history(key)
            .is_some_and(|(history, complete)| {
                complete && history.reference_may_be_replaced(path, reads)
            })
    }
}
