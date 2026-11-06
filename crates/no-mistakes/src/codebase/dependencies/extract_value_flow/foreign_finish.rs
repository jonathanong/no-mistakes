use super::*;

impl ImportCollector {
    pub(in super::super::super::super) fn finish_foreign_receiver_writes(&mut self) {
        let Some(mut writes) = self.foreign_receiver_writes.take() else {
            return;
        };
        if writes.work.exhausted() {
            self.value_flow.incomplete_reason = Some(super::super::budget::REASON.into());
            return;
        }
        for (_, path, ..) in writes.writes.keys() {
            if writes.work.charge(1 + path.len()).is_none() {
                self.value_flow.incomplete_reason = Some(super::super::budget::REASON.into());
                return;
            }
        }
        writes.finishing = true;
        let mut pending: Vec<_> = std::mem::take(&mut writes.writes).into_iter().collect();
        pending.sort_by(|(left, _), (right, _)| left.cmp(right));
        self.foreign_receiver_writes = Some(writes);
        #[cfg(feature = "test-instrumentation")]
        count(1, pending.len());
        let mut providers = fx_map();
        let empty_origins = ReceiverBindingOrigins::default();
        let mut arguments = None;
        let mut aliases = None;
        for ((key, path, (owner, acquired), assigned, has_assignment, possible), mut write) in
            pending
        {
            let selected = providers
                .entry((key.clone(), path.clone()))
                .or_insert_with(|| {
                    let mut possible = Vec::new();
                    let mut members = self
                        .prior_member_mutations
                        .foreign_member_provider_references(
                            &key,
                            &path,
                            &mut self.foreign_receiver_writes.as_mut().unwrap().work,
                        )?;
                    if members.iter().any(|((receiver, _, _), _, _)| {
                        self.value_parameter_indexes
                            .get(&receiver.0 .0)
                            .is_some_and(|names| names.contains_key(&receiver.0 .1))
                    }) {
                        possible = arguments
                            .get_or_insert_with(|| {
                                arguments::MemberArguments::new(self.parameter_call_origins())
                            })
                            .expand(
                                &mut members,
                                &mut self.foreign_receiver_writes.as_mut().unwrap().work,
                            )?;
                    }
                    let origins = self
                        .receiver_only_bindings
                        .as_deref()
                        .unwrap_or(&empty_origins);
                    let work = &mut self.foreign_receiver_writes.as_mut().unwrap().work;
                    let possible = if possible.is_empty() {
                        None
                    } else {
                        Some(origins.foreign_provider_index(&key, &path, possible, work)?)
                    };
                    Some((
                        origins.foreign_provider_index(&key, &path, members, work)?,
                        possible,
                    ))
                });
            let Some((index, possible_index)) = selected else {
                break;
            };
            if self.charge_foreign_work(1).is_none() {
                break;
            }
            // A direct writer may acquire its own later provider on another call.
            // A nested closure retains its enclosing owner's saved acquisition.
            let references = if possible {
                Vec::new()
            } else {
                if self
                    .charge_foreign_work(index.reference_count() + index.module_references.len())
                    .is_none()
                {
                    break;
                }
                index.references(
                    (owner != acquired).then_some(acquired),
                    write.metadata.foreign_before,
                )
            };
            // The declaration order cannot establish an exact replacement or
            // a fresh-object certificate. Preserve the writer, member and scope.
            write.metadata.fresh = false;
            write.metadata.unconditional = false;
            write.metadata.region = None;
            write.metadata.phase = None;
            write.metadata.foreign_before = usize::MAX;
            let info = MutationInfo {
                metadata: write.metadata,
                assigned_member: assigned.as_deref(),
                completed_at: write.completed_at,
                has_assignment,
                proof: None,
            };
            if let Some(possible) = possible_index {
                self.replay_possible_foreign(
                    possible,
                    (owner != acquired).then_some(acquired),
                    info,
                );
            }
            if possible {
                self.replay_possible_foreign(index, (owner != acquired).then_some(acquired), info);
            } else {
                self.replay_foreign(index, references, owner, info, &mut aliases);
            }
            if self.foreign_work_exhausted() {
                break;
            }
        }
        if self
            .foreign_receiver_writes
            .as_ref()
            .unwrap()
            .work
            .exhausted()
        {
            self.value_flow.incomplete_reason = Some(super::super::budget::REASON.into());
        }
        self.foreign_receiver_writes = None;
    }
    pub(in super::super::super::super) fn finish_bounded_value_flow(&mut self) {
        if let Some(reason) = self.value_flow.incomplete_reason.take() {
            self.value_flow = ValueFlowFacts {
                incomplete_reason: Some(reason),
                ..Default::default()
            };
        } else {
            self.finish_default_mutations();
            finish_callable_assignment_facts(self);
        }
    }
    pub(in super::super::super) fn charge_foreign_work(&mut self, amount: usize) -> Option<()> {
        match self
            .foreign_receiver_writes
            .as_mut()
            .filter(|writes| writes.budgeting || writes.finishing)
        {
            Some(writes) => writes.work.charge(amount),
            None => Some(()),
        }
    }
    pub(in super::super::super) fn remaining_foreign_work(&self) -> Option<usize> {
        self.foreign_receiver_writes
            .as_ref()
            .filter(|writes| writes.budgeting || writes.finishing)
            .map(|writes| writes.work.remaining())
    }
    pub(in super::super::super) fn foreign_work_exhausted(&self) -> bool {
        self.foreign_receiver_writes
            .as_ref()
            .is_some_and(|writes| writes.work.exhausted())
    }
}
