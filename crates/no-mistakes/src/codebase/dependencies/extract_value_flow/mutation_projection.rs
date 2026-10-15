use super::*;

pub(super) type Affected = FxHashMap<usize, FxHashSet<String>>;
pub(super) type PendingMutation = (nested::ReceiverWrite, bool, bool, bool);
#[derive(Clone, Copy)]
pub(super) struct MutationInfo<'a> {
    pub(super) metadata: mutation_metadata::MutationMetadata,
    pub(super) assigned_member: Option<&'a str>,
    pub(super) completed_at: u32,
    pub(super) has_assignment: bool,
    pub(super) proof: Option<&'a mutable_deferred::DeferredProof>,
}
impl ImportCollector {
    pub(super) fn project_member_mutation(
        &mut self,
        mut pending: Vec<PendingMutation>,
        mut assigned: Option<(Option<String>, u32)>,
        info: MutationInfo<'_>,
        affected: &mut Affected,
    ) {
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
            // Direct component aliases have no value projection. Keep their
            // canonical proof lookup cold, as in the original traversal.
            let projects = self.has_mutable_receiver_binding(key.0, &key.1)
                || self
                    .value_binding_indexes
                    .get(&key.0)
                    .and_then(|names| names.get(key.1.as_str()))
                    .into_iter()
                    .flatten()
                    .any(|index| {
                        let value = &self.value_flow.bindings[*index].value;
                        !matches!(value, ValueExpression::Named { name, .. } if !name.contains('.'))
                            || self.references_mutable_receiver(value)
                    });
            let replaced = projects && self.mutation_reference_replaced(&key, &path, &reads, info);
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
                if replaced {
                    continue;
                }
                nested::write_receiver_references(value, &path, &reads, &mut references);
            }
            if !self.defer_mutable_projection(&key, (&path, &reads, root_read), info, replaced)
                && !replaced
            {
                self.append_mutable_receiver_references_unchecked(
                    &key,
                    (&path, &reads, root_read),
                    &mut references,
                    info.metadata.foreign_before,
                );
            }
            pending.extend(
                references
                    .into_iter()
                    .map(|reference| (reference, false, true, true)),
            );
        }
    }
}
