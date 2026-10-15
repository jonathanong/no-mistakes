use super::*;
use mutation_history::{ValueAssignedMutation, ValueAssignedWriter, ValueMutationHistory};

pub(super) struct AssignmentTiming {
    pub(super) started_at: u32,
    pub(super) context: Option<CallableId>,
    pub(super) completed: ValueAssignedMutation,
}

impl ValueAssignedMutation {
    /// A single module/callable writer, or no unique writer for mixed writes.
    pub(in crate::codebase::dependencies) fn single_context(&self) -> Option<Option<CallableId>> {
        (self.writer != ValueAssignedWriter::Mixed).then(|| self.writer.context())
    }
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
        timing: AssignmentTiming,
        propagated: bool,
        replacement: Option<bool>,
        proof: (Option<u32>, Option<identity_metadata::FreshCapture>),
    ) {
        let AssignmentTiming {
            started_at,
            context,
            completed: initial,
        } = timing;
        let (region, capture) = proof;
        self.has_assignments = true;
        let mut node = self;
        // Coarse histories use the write start; exact leaves and replacement
        // proofs use completion. Share only their path descent, not their clock.
        for segment in path {
            node.record_summary(started_at, context);
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
            node = node.record_child(segment);
        }
        node.record_summary(started_at, context);
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
        let phases = self
            .fresh_receiver_effects
            .phases
            .as_ref()
            .map(|phases| phases.borrow());
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
                        phases
                            .as_deref()
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
