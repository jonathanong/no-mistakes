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
        if !member
            || self
                .receiver_conditional_region
                .is_some_and(|(_, admitted)| admitted != offset)
        {
            self.receiver_conditional_region = None;
        }
        if !member
            && (self.prior_member_mutations.requires_proof(scope, base)
                || self.has_mutable_receiver_binding(scope, base))
        {
            self.fresh_receiver_effects.rebind(scope, base);
        }
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
            // A nested alias writes the same member on its original receiver.
            // Keep exact assignment metadata alongside every propagated path.
            let assigned_member = match receiver {
                Some(path) => path.last().and_then(Option::as_deref),
                None => name.rsplit_once('.').map(|(_, member)| member),
            };
            let mut assigned = parts.pop().map(|member| (member, completed_at));
            let has_assignment = assigned.is_some();
            let acquired_at = self.receiver_acquisition_offset(offset);
            let reads = vec![Some((acquired_at, self.current_function_id())); parts.len()];
            let mut pending = vec![(
                (
                    ((scope, base.to_string()), parts),
                    reads,
                    (acquired_at, self.current_function_id()),
                ),
                false,
                true,
                false,
            )];
            let mut traversal = receiver_traversal::ReceiverTraversal::default();
            while let Some((((key, path), reads, root_read), exiting, remember, propagated)) =
                pending.pop()
            {
                let version = self.mutable_receiver_version(&key, root_read);
                if exiting {
                    traversal.exit(&key, version);
                    continue;
                }
                if remember {
                    let exact = assigned.take().or_else(|| {
                        has_assignment.then(|| (assigned_member.map(str::to_string), completed_at))
                    });
                    self.remember_member_mutation(&key, &path, &reads, exact, offset, propagated);
                }
                // Version-qualified mutable roots may revisit an earlier saved
                // root without conflating it with the replacement's identity.
                let Some(reads) = traversal.enter(&key, &path, reads, version) else {
                    continue;
                };
                #[cfg(feature = "test-instrumentation")]
                {
                    self.value_flow.reference_path_visits += 1;
                }
                pending.push((
                    ((key.clone(), Vec::new()), Vec::new(), root_read),
                    true,
                    false,
                    false,
                ));
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
                        ((target.clone(), path.clone()), reads.clone(), root_read),
                        false,
                        false,
                        propagated,
                    ));
                }
                let mut references = Vec::new();
                for value in self
                    .value_binding_indexes
                    .get(&key.0)
                    .and_then(|names| names.get(key.1.as_str()))
                    .into_iter()
                    .flatten()
                    .map(|index| &self.value_flow.bindings[*index].value)
                {
                    // Direct aliases are handled by the component links at finalization.
                    if matches!(value, ValueExpression::Named { name, .. } if !name.contains('.'))
                        && !self.references_mutable_receiver(value)
                    {
                        continue;
                    }
                    if self.reference_path_may_be_replaced(&key, &path, &reads) {
                        continue;
                    }
                    nested::write_receiver_references(value, &path, &reads, &mut references);
                }
                self.append_mutable_receiver_references(
                    &key,
                    (&path, &reads, root_read),
                    &mut references,
                );
                pending.extend(
                    references
                        .into_iter()
                        .map(|reference| (reference, false, true, true)),
                );
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
