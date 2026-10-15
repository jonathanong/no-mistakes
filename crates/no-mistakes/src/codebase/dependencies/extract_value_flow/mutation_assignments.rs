use super::*;
use mutation_history::{ValueAssignedMutation, ValueAssignedWriter, ValueMutationHistory};

impl ValueAssignedMutation {
    pub(super) fn merge(&mut self, source: Self) {
        self.offset = self.offset.min(source.offset);
        self.unconditional_offset = self
            .unconditional_offset
            .into_iter()
            .chain(source.unconditional_offset)
            .min();
        if self.writer != source.writer {
            self.writer = ValueAssignedWriter::Mixed;
        }
    }
    pub(super) fn invalidated(&self, reader: &impl Fn(Option<CallableId>) -> Option<u32>) -> bool {
        self.writer == ValueAssignedWriter::Mixed
            || reader(self.writer.context()).is_none_or(|offset| self.offset <= offset)
    }
}
impl ValueMutationHistory {
    pub(super) fn reference_may_be_replaced(
        &self,
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        region: Option<u32>,
        effects: Option<&fresh_receiver_effects::FreshReceiverEffects>,
        phases: Option<active_reference_phases::PhaseCursor<'_>>,
    ) -> bool {
        reference_identity::reference_replaced(self, path, reads, region, effects, phases)
    }

    pub(super) fn record_assignment(
        &mut self,
        path: &[Option<String>],
        member: Option<String>,
        initial: ValueAssignedMutation,
        propagated: bool,
        replacement: Option<bool>,
        region: Option<u32>,
        capture: Option<identity_metadata::FreshCapture>,
    ) {
        self.has_assignments = true;
        let mut node = self;
        for segment in path {
            if segment.is_none() && replacement.is_some() {
                node.assigned
                    .get_or_insert_with(Default::default)
                    .replacements
                    .get_or_insert_with(Default::default)
                    .identity
                    .get_or_insert_with(Default::default)
                    .unknown
                    .record(initial.offset, initial.writer);
            }
            node = match segment {
                Some(member) => node.members.entry(member.clone()).or_default(),
                None => node.dynamic.get_or_insert_with(Default::default),
            };
        }
        let assigned = node.assigned.get_or_insert_with(Default::default);
        if let Some(fresh) = replacement {
            assigned
                .replacements
                .get_or_insert_with(Default::default)
                .record(
                    member.as_deref(),
                    initial.offset,
                    fresh,
                    initial.unconditional_offset.is_some(),
                );
        }
        if let (Some(true), Some(region), Some(member)) = (replacement, region, member.as_deref()) {
            assigned
                .replacements
                .get_or_insert_with(Default::default)
                .record_region(region, member, initial.offset);
        }
        if replacement == Some(true) && (initial.unconditional_offset.is_some() || region.is_some())
        {
            let identity = assigned
                .replacements
                .get_or_insert_with(Default::default)
                .identity
                .get_or_insert_with(Default::default);
            if let Some(capture) = capture {
                identity.capture(initial.offset, capture);
            } else {
                identity.captures.insert(initial.offset, None);
            }
        }
        if propagated {
            assigned
                .propagated
                .get_or_insert_with(Default::default)
                .record(member, initial);
        } else {
            assigned.record(member, initial);
        }
    }

    pub fn assignment_invalidated(
        &self,
        projection: &[&str],
        offset: u32,
        context: Option<CallableId>,
        external: bool,
    ) -> bool {
        self.assignment_invalidated_reads(projection, &|writer| {
            (!external && writer == context).then_some(offset)
        })
    }
    pub fn assignment_invalidated_reads(
        &self,
        projection: &[&str],
        reader: &impl Fn(Option<CallableId>) -> Option<u32>,
    ) -> bool {
        self.assignment_invalidated_owned_reads(projection, reader, &|| None)
    }
    pub fn assignment_invalidated_owned_reads(
        &self,
        projection: &[&str],
        reader: &impl Fn(Option<CallableId>) -> Option<u32>,
        owner: &impl Fn() -> Option<CallableId>,
    ) -> bool {
        let Some((member, prefix)) = projection.split_first() else {
            return false;
        };
        if prefix.is_empty() {
            return self
                .assigned
                .as_ref()
                .is_some_and(|members| members.invalidated(member, reader, owner));
        }

        let (next, _) = prefix.split_last().expect("nonempty prefix");
        let child_matches = |child: &Self| {
            child.assignment_invalidated_owned_reads(
                &projection[..projection.len() - 1],
                reader,
                owner,
            )
        };
        self.members.get(*next).is_some_and(child_matches)
            || self.dynamic.as_deref().is_some_and(child_matches)
    }
}

impl ImportCollector {
    pub(super) fn reference_path_may_be_replaced(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
    ) -> bool {
        if path.is_empty() || self.fresh_receiver_effects.rebound(key.0, &key.1) {
            return false;
        }
        self.prior_member_mutations
            .history(key)
            .is_some_and(|(history, complete)| {
                complete
                    && history.reference_may_be_replaced(
                        path,
                        reads,
                        self.receiver_conditional_region
                            .map(|(token, _)| token.get()),
                        Some(&self.fresh_receiver_effects),
                        self.fresh_receiver_effects
                            .phases
                            .as_ref()
                            .filter(|_| {
                                self.destructuring_phase_owner == Some(self.current_function_id())
                            })
                            .map(|phases| {
                                phases.cursor(
                                    &self.prior_member_mutations,
                                    key,
                                    self.current_function_id(),
                                )
                            }),
                    )
            })
    }
}
