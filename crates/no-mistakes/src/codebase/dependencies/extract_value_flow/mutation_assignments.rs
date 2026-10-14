use super::*;
use mutation_history::{ValueAssignedMutation, ValueAssignedMutations, ValueMutationHistory};

impl ValueAssignedMutation {
    fn merge(&mut self, source: Self) {
        self.offset = self.offset.min(source.offset);
        self.multiple_contexts |= source.multiple_contexts || self.context != source.context;
        if self.multiple_contexts {
            self.context = None;
        }
    }
    fn invalidated(&self, reader: &impl Fn(Option<CallableId>) -> Option<u32>) -> bool {
        self.multiple_contexts || reader(self.context).is_none_or(|offset| self.offset <= offset)
    }
}
impl ValueAssignedMutations {
    pub(super) fn merge(&mut self, mut source: Self) {
        if source.members.len() > self.members.len() {
            std::mem::swap(&mut self.members, &mut source.members);
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.merge_reinsertions += source.merge_reinsertions + source.members.len();
        }
        for (member, source) in source.members {
            self.members.entry(member).or_insert(source).merge(source);
        }
        if let Some(source) = source.dynamic {
            self.dynamic.get_or_insert(source).merge(source);
        }
    }
}

impl ValueMutationHistory {
    pub(super) fn record_assignment(
        &mut self,
        path: &[Option<String>],
        member: Option<String>,
        offset: u32,
        context: Option<CallableId>,
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
        let initial = ValueAssignedMutation {
            offset,
            context,
            multiple_contexts: false,
        };
        let write = match member {
            Some(member) => assigned.members.entry(member).or_insert(initial),
            None => assigned.dynamic.get_or_insert(initial),
        };
        write.merge(initial);
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
        let Some((member, prefix)) = projection.split_first() else {
            return false;
        };
        if prefix.is_empty() {
            return self.assigned.as_ref().is_some_and(|members| {
                [members.members.get(*member), members.dynamic.as_ref()]
                    .into_iter()
                    .flatten()
                    .any(|write| write.invalidated(reader))
            });
        }
        let (next, _) = prefix.split_last().expect("nonempty prefix");
        let child_matches = |child: &Self| {
            child.assignment_invalidated_reads(&projection[..projection.len() - 1], reader)
        };
        self.members.get(*next).is_some_and(child_matches)
            || self.dynamic.as_deref().is_some_and(child_matches)
    }
}
