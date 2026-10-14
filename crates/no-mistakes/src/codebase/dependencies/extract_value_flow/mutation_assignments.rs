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
    ) -> bool {
        let mut node = self;
        for (member, read) in path.iter().zip(reads) {
            let Some((offset, context)) = read else {
                return false;
            };
            let Some(member) = member else {
                return false;
            };
            if node
                .assigned
                .as_ref()
                .is_some_and(|assigned| assigned.definitely_replaced(member, *offset, *context))
            {
                return true;
            }
            let Some(child) = node.members.get(member) else {
                return false;
            };
            node = child;
        }
        false
    }

    pub(super) fn record_assignment(
        &mut self,
        path: &[Option<String>],
        member: Option<String>,
        initial: ValueAssignedMutation,
        propagated: bool,
        replacement: Option<bool>,
    ) {
        self.has_assignments = true;
        let mut node = self;
        for segment in path {
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
