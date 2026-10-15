use super::*;

/// The common single-writer case shares its aggregate and context projection.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ValuePropagatedMutations {
    aggregate: ValueAssignedMutations,
    #[cfg(feature = "test-instrumentation")]
    records: usize,
    single: Option<Option<CallableId>>,
    contexts: FxHashMap<Option<CallableId>, ValueAssignedMutations>,
}
impl ValuePropagatedMutations {
    /// All propagated writes, regardless of their callable owner.
    pub fn aggregate(&self) -> &ValueAssignedMutations {
        &self.aggregate
    }
    /// Propagated writes belonging to this exact module or callable owner.
    pub fn context(&self, owner: Option<CallableId>) -> Option<&ValueAssignedMutations> {
        match self.single {
            Some(single) => (single == owner).then_some(&self.aggregate),
            None => self.contexts.get(&owner),
        }
    }
    fn promote(&mut self) {
        if let Some(owner) = self.single.take() {
            self.contexts.insert(owner, self.aggregate.clone());
        }
    }
    pub(in super::super) fn record(
        &mut self,
        member: Option<String>,
        write: ValueAssignedMutation,
    ) {
        let owner = write.writer.context();
        if self.single == Some(owner) || (self.single.is_none() && self.contexts.is_empty()) {
            self.single = Some(owner);
            #[cfg(feature = "test-instrumentation")]
            {
                self.records += 1;
            }
            self.aggregate.record(member, write);
        } else {
            self.promote();
            #[cfg(feature = "test-instrumentation")]
            {
                self.records += 2;
            }
            self.aggregate.record(member.clone(), write);
            self.contexts
                .entry(owner)
                .or_default()
                .record(member, write);
        }
    }
    pub(in super::super) fn merge(&mut self, source: Self) {
        #[cfg(feature = "test-instrumentation")]
        let records = self.records + source.records;
        if source.single.is_none() && source.contexts.is_empty() {
            return;
        }
        if self.single.is_none() && self.contexts.is_empty() {
            *self = source;
        } else if self.single.is_some() && self.single == source.single {
            self.aggregate.merge(source.aggregate);
        } else {
            self.promote();
            if let Some(owner) = source.single {
                self.aggregate.merge(source.aggregate.clone());
                self.contexts
                    .entry(owner)
                    .or_default()
                    .merge(source.aggregate);
            } else {
                self.aggregate.merge(source.aggregate);
                for (owner, writes) in source.contexts {
                    self.contexts.entry(owner).or_default().merge(writes);
                }
            }
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.records = records;
        }
    }
}
#[cfg(test)]
#[path = "propagated_mutation_tests.rs"]
mod tests;
// Preserve the logical fact dump: storage sharing must not alter debug reports.
impl std::fmt::Debug for ValuePropagatedMutations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct Contexts<'a>(&'a ValuePropagatedMutations);
        impl std::fmt::Debug for Contexts<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.0.single {
                    Some(owner) => f.debug_map().entry(&owner, &self.0.aggregate).finish(),
                    None => std::fmt::Debug::fmt(&self.0.contexts, f),
                }
            }
        }
        f.debug_struct("ValuePropagatedMutations")
            .field("aggregate", &self.aggregate)
            .field("contexts", &Contexts(self))
            .finish()
    }
}
