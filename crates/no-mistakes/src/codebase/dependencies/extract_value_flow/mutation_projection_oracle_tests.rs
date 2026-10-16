use super::*;

impl ImportCollector {
    pub(in super::super) fn project_member_mutation_legacy<R: nested::RootCoordinate>(
        &mut self,
        mut pending: Vec<PendingMutation<R>>,
        mut assigned: Option<(Option<String>, u32)>,
        info: MutationInfo<'_>,
        affected: &mut Affected,
    ) {
        let mut traversal = receiver_traversal::ReceiverTraversal::default();
        #[cfg(feature = "test-instrumentation")]
        let mut binding_visits = 0;
        while let Some((((key, path), reads, root_read), exiting, remember, propagated)) =
            pending.pop()
        {
            if exiting {
                self.finish_projected_receiver(&mut traversal, &key, root_read, propagated);
                continue;
            }
            if remember {
                let exact = assigned.take().or_else(|| {
                    info.has_assignment
                        .then(|| (info.assigned_member.map(str::to_string), info.completed_at))
                });
                self.remember_member_mutation(
                    &key,
                    &path,
                    &reads,
                    exact,
                    mutation_metadata::Tracking {
                        metadata: &info.metadata,
                        propagated,
                        root_read: root_read.read(),
                    },
                );
            }
            let terminal = root_read.read().is_some()
                && !self.prior_member_mutations.has_assigned_origins()
                && self.mutation_reference_is_terminal(&key);
            #[cfg(feature = "test-instrumentation")]
            if terminal {
                self.value_flow.reference_terminal_paths += 1;
            }
            // RHS facts are another possible successor source. Keep their
            // canonical projection whenever that request-owned index exists.
            if terminal {
                affected.insert(key.0, &key.1);
                continue;
            }
            let version = self.projected_root_version(&key, root_read);
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_successor_bookkeeping += 1;
            }
            let mut assigned_origin = if self.prior_member_mutations.has_assigned_origins() {
                let canonical = self.assignment_rhs_parent_identities(&key, &path, &reads, version);
                self.prior_member_mutations
                    .assigned_origin(&key, &path, &reads, version, &canonical)
            } else {
                None
            };
            let assigned_exit = assigned_origin.is_some();
            let entered = match assigned_origin.as_mut() {
                Some(selection) => traversal.enter_assigned(
                    &key,
                    &path,
                    reads,
                    version,
                    std::mem::take(&mut selection.receipts),
                ),
                None if root_read.read().is_none() => traversal.enter_ordinary(&key, &path, reads),
                None => traversal.enter(&key, &path, reads, version),
            };
            let Some(reads) = entered else { continue };
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_path_visits += 1;
            }
            pending.push((
                ((key.clone(), Vec::new()), Vec::new(), root_read),
                true,
                false,
                assigned_exit,
            ));
            affected.insert(key.0, &key.1);
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
            let mut references =
                reference_output::ReferenceOutput::<nested::ReceiverWrite<R>>::default();
            if let Some(selection) = &assigned_origin {
                selection.append_references(&path, &reads, &mut references);
            }
            let assigned_capture =
                assigned_origin.and_then(assigned_receiver_origins::Selection::captured);
            let has_assigned_origin = assigned_capture.is_some_and(|(prefix, capture)| {
                self.assignment_rhs_reference_intact(&key, (&path, &reads, prefix, &capture), info)
            });
            // Direct component aliases have no value projection. Keep their
            // canonical proof lookup cold, as in the original traversal.
            let first_projected = self.first_projected_receiver(
                &key,
                root_read.read().is_some(),
                #[cfg(feature = "test-instrumentation")]
                &mut binding_visits,
            );
            let projects = version.is_some() || first_projected.is_some();
            let replaced = projects && self.mutation_reference_replaced(&key, &path, &reads, info);
            // The eligibility prefix contains only direct aliases. They have
            // already been handled by component links and need no second visit.
            if let Some(first) = first_projected.filter(|_| !replaced && !has_assigned_origin) {
                for value in self
                    .value_binding_indexes
                    .get(&key.0)
                    .and_then(|names| names.get(key.1.as_str()))
                    .map_or(&[][..], Vec::as_slice)[first..]
                    .iter()
                    .map(|index| {
                        #[cfg(feature = "test-instrumentation")]
                        {
                            binding_visits += 1;
                        }
                        &self.value_flow.bindings[*index].value
                    })
                {
                    if matches!(value, ValueExpression::Named { name, .. } if !name.contains('.'))
                        && (root_read.read().is_none() || !self.references_mutable_receiver(value))
                    {
                        continue;
                    }
                    nested::write_receiver_references(value, &path, &reads, &mut references);
                }
            }
            if let (Some(read), Some(version)) = (root_read.read(), version) {
                if !self.defer_mutable_projection(
                    &key,
                    (&path, &reads, read),
                    info,
                    replaced,
                    version,
                ) && !replaced
                {
                    #[cfg(feature = "test-instrumentation")]
                    {
                        self.value_flow.reference_root_probes += 1;
                    }
                    self.append_mutable_receiver_references_unchecked(
                        &key,
                        (&path, &reads, read),
                        &mut references,
                        info.metadata.foreign_before,
                        version,
                    );
                }
            }
            #[cfg(feature = "test-instrumentation")]
            references.record_counts(&mut self.value_flow);
            pending.extend(
                references
                    .into_iter()
                    .map(|reference| (reference, false, true, true)),
            );
        }
        #[cfg(feature = "test-instrumentation")]
        {
            traversal.record_counts(&mut self.value_flow);
            self.value_flow.reference_binding_visits += binding_visits;
            self.value_flow.reference_pending_capacity += pending.capacity();
            self.value_flow.reference_pending_bytes +=
                pending.capacity() * std::mem::size_of::<PendingMutation<R>>();
        }
    }
}
