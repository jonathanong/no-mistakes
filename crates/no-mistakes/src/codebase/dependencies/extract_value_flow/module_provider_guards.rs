use super::*;
use mutation_history::{ValueAssignedMutation, ValueAssignedMutations, ValueMutationHistory};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuardedWrite {
    timing: ValueAssignedMutation,
    provider: Arc<ModuleProviderTarget>,
    propagated: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ModuleAssignedMutations {
    members: FxHashMap<Option<String>, Vec<GuardedWrite>>,
    pub(super) aggregate: ValueAssignedMutations,
}
impl ModuleAssignedMutations {
    pub(super) fn merge(&mut self, source: Self) {
        self.aggregate.merge(source.aggregate);
        for (member, writes) in source.members {
            self.members.entry(member).or_default().extend(writes);
        }
    }
    pub(crate) fn active_summary(
        &self,
        active: impl Fn(Option<CallableId>, &ModuleProviderTarget) -> bool,
    ) -> ValueAssignedMutations {
        let mut summary = ValueAssignedMutations::default();
        for (member, writes) in &self.members {
            for write in writes {
                if active(write.timing.writer.context(), &write.provider) {
                    record(&mut summary, member.clone(), write.timing, write.propagated);
                }
            }
        }
        summary
    }
}
fn record(
    summary: &mut ValueAssignedMutations,
    member: Option<String>,
    timing: ValueAssignedMutation,
    propagated: bool,
) {
    if propagated {
        summary
            .propagated
            .get_or_insert_with(Default::default)
            .record(member, timing);
    } else {
        summary.record(member, timing);
    }
}

impl ValueMutationHistory {
    pub(crate) fn module_writer_contexts(&self, contexts: &mut FxHashSet<CallableId>) {
        let mut pending = vec![self];
        while let Some(history) = pending.pop() {
            if let Some(module) = history
                .assigned
                .as_ref()
                .and_then(|assigned| assigned.module.as_ref())
            {
                contexts.extend(
                    module
                        .members
                        .values()
                        .flatten()
                        .filter_map(|write| write.timing.single_context().flatten()),
                );
            }
            pending.extend(history.members.values());
            pending.extend(history.dynamic.as_deref());
        }
    }
    pub(super) fn record_module_assignment(
        &mut self,
        path: &[Option<String>],
        member: Option<String>,
        timing: ValueAssignedMutation,
        provider: Arc<ModuleProviderTarget>,
        propagated: bool,
    ) {
        self.has_assignments = true;
        let mut node = self;
        for segment in path {
            node = node.record_child(segment);
        }
        let module = node
            .assigned
            .get_or_insert_with(Default::default)
            .module
            .get_or_insert_with(Default::default);
        record(&mut module.aggregate, member.clone(), timing, propagated);
        module
            .members
            .entry(member)
            .or_default()
            .push(GuardedWrite {
                timing,
                provider,
                propagated,
            });
    }
    pub(crate) fn assignment_invalidated_guarded_reads(
        &self,
        projection: &[&str],
        reader: &impl Fn(Option<CallableId>) -> Option<u32>,
        owner: &impl Fn() -> Option<CallableId>,
        active: &impl Fn(&ModuleAssignedMutations) -> Arc<ValueAssignedMutations>,
    ) -> bool {
        let Some((member, prefix)) = projection.split_first() else {
            return false;
        };
        if prefix.is_empty() {
            return self.assigned.as_ref().is_some_and(|members| {
                members.unguarded_invalidated(member, reader, owner)
                    || members
                        .module
                        .as_ref()
                        .is_some_and(|module| active(module).invalidated(member, reader, owner))
            });
        }
        let (next, _) = prefix.split_last().expect("nonempty prefix");
        let child_matches = |child: &Self| {
            child.assignment_invalidated_guarded_reads(
                &projection[..projection.len() - 1],
                reader,
                owner,
                active,
            )
        };
        self.members.get(*next).is_some_and(child_matches)
            || self.dynamic.as_deref().is_some_and(child_matches)
    }
}

impl ImportCollector {
    pub(super) fn remember_module_provider_mutation(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        assigned: Option<(Option<String>, u32)>,
        tracking: &mutation_metadata::Tracking<'_>,
    ) -> bool {
        let Some(provider) = self.active_module_provider.as_ref() else {
            return false;
        };
        let (member, offset) = assigned.unwrap_or((None, tracking.metadata.offset));
        let (history, _) = self.prior_member_mutations.history_mut(key);
        history.record_module_assignment(
            path,
            member,
            ValueAssignedMutation {
                offset,
                unconditional_offset: None,
                writer: mutation_history::ValueAssignedWriter::single(tracking.metadata.context),
            },
            Arc::clone(provider),
            tracking.propagated,
        );
        true
    }
}

#[cfg(test)]
#[path = "module_provider_guards_tests.rs"]
mod tests;
