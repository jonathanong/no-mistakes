use super::*;

pub(super) type Affected = FxHashMap<usize, FxHashSet<String>>;
pub(super) type PendingMutation<R = nested::RootRead> =
    (nested::ReceiverWrite<R>, bool, bool, bool);
#[derive(Clone, Copy)]
pub(super) struct MutationInfo<'a> {
    pub(super) metadata: mutation_metadata::MutationMetadata,
    pub(super) assigned_member: Option<&'a str>,
    pub(super) completed_at: u32,
    pub(super) has_assignment: bool,
    pub(super) proof: Option<&'a mutable_deferred::DeferredProof>,
}
impl ImportCollector {
    pub(super) fn project_member_mutation<R: nested::RootCoordinate>(
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
                if root_read.read().is_some() && traversal.exit_single_version(&key) {
                    continue;
                }
                let version = root_read.read().and_then(|read| {
                    #[cfg(feature = "test-instrumentation")]
                    {
                        self.value_flow.reference_root_probes += 1;
                    }
                    self.mutable_receiver_version(&key, read)
                });
                traversal.exit(&key, version);
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
                    &info.metadata,
                    propagated,
                );
            }
            let terminal = root_read.read().is_some() && self.mutation_reference_is_terminal(&key);
            #[cfg(feature = "test-instrumentation")]
            if terminal {
                self.value_flow.reference_terminal_paths += 1;
            }
            // No descriptor, component link, or mutable origin can enqueue a
            // successor. Mutation recording above and invalidation still apply.
            if terminal {
                affected.entry(key.0).or_default().insert(key.1.clone());
                continue;
            }
            let version = root_read.read().and_then(|read| {
                #[cfg(feature = "test-instrumentation")]
                {
                    self.value_flow.reference_root_probes += 1;
                }
                self.mutable_receiver_version(&key, read)
            });
            // Version-qualified mutable roots may revisit an earlier saved
            // root without conflating it with the replacement's identity.
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_successor_bookkeeping += 1;
            }
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
            // Direct component aliases have no value projection. Keep their
            // canonical proof lookup cold, as in the original traversal.
            let first_projected = self
                .value_binding_indexes
                .get(&key.0)
                .and_then(|names| names.get(key.1.as_str()))
                .map_or(&[][..], Vec::as_slice)
                .iter()
                .position(|index| {
                    #[cfg(feature = "test-instrumentation")]
                    {
                        binding_visits += 1;
                    }
                    let value = &self.value_flow.bindings[*index].value;
                    !matches!(value, ValueExpression::Named { name, .. } if !name.contains('.'))
                        || (root_read.read().is_some() && self.references_mutable_receiver(value))
                });
            let projects = (root_read.read().is_some()
                && self.has_mutable_receiver_binding(key.0, &key.1))
                || first_projected.is_some();
            let replaced = projects && self.mutation_reference_replaced(&key, &path, &reads, info);
            // The eligibility prefix contains only direct aliases. They have
            // already been handled by component links and need no second visit.
            if let Some(first) = first_projected.filter(|_| !replaced) {
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
            if let Some(read) = root_read.read() {
                #[cfg(feature = "test-instrumentation")]
                {
                    self.value_flow.reference_root_probes += 1;
                }
                if !self.defer_mutable_projection(&key, (&path, &reads, read), info, replaced)
                    && !replaced
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
                    );
                }
            }
            pending.extend(
                references
                    .into_iter()
                    .map(|reference| (reference, false, true, true)),
            );
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_version_table_capacity += traversal.version_table_capacity();
            self.value_flow.reference_binding_visits += binding_visits;
            self.value_flow.reference_pending_capacity += pending.capacity();
            self.value_flow.reference_pending_bytes +=
                pending.capacity() * std::mem::size_of::<PendingMutation<R>>();
        }
    }
}
