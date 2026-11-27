use super::*;
use mutation_history::{ValueAssignedMutation, ValueAssignedMutations};

impl ValueAssignedMutations {
    pub(super) fn merge(&mut self, mut source: Self) {
        if let Some(source) = source.module.take() {
            self.module
                .get_or_insert_with(Default::default)
                .merge(*source);
        }
        if let Some(early) = source.early.take() {
            self.early
                .get_or_insert_with(Default::default)
                .merge(*early);
        }
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
        if let Some(source) = source.replacements {
            if let Some(target) = &mut self.replacements {
                target.merge(*source);
            } else {
                self.replacements = Some(source);
            }
        }
        if let Some(source) = source.propagated {
            if let Some(target) = &mut self.propagated {
                target.merge(*source);
            } else {
                self.propagated = Some(source);
            }
        }
    }

    pub(super) fn record(&mut self, member: Option<String>, initial: ValueAssignedMutation) {
        let write = match member {
            Some(member) => self.members.entry(member).or_insert(initial),
            None => self.dynamic.get_or_insert(initial),
        };
        write.merge(initial);
    }

    pub(super) fn invalidated(
        &self,
        member: &str,
        reader: &impl Fn(Option<CallableId>) -> Option<u32>,
        owner: &impl Fn() -> Option<CallableId>,
    ) -> bool {
        self.unguarded_invalidated(member, reader, owner)
            || self
                .module
                .as_ref()
                .is_some_and(|module| module.aggregate.invalidated(member, reader, owner))
    }

    pub(super) fn unguarded_invalidated(
        &self,
        member: &str,
        reader: &impl Fn(Option<CallableId>) -> Option<u32>,
        owner: &impl Fn() -> Option<CallableId>,
    ) -> bool {
        self.direct_invalidated(member, reader)
            || self
                .early
                .as_ref()
                .is_some_and(|early| early.invalidated(member, reader, self))
            || self.propagated.as_ref().is_some_and(|propagated| {
                let owner = owner();
                // Module writes still use the caller's recorded acquisition.
                // Only unrelated callable writers are excluded for defaults.
                if owner.is_some()
                    && propagated
                        .context(None)
                        .is_some_and(|writes| writes.direct_invalidated(member, reader))
                {
                    return true;
                }
                let selected = match owner {
                    Some(owner) => propagated.context(Some(owner)),
                    None => Some(propagated.aggregate()),
                };
                selected.is_some_and(|selected| selected.direct_invalidated(member, reader))
            })
    }

    fn direct_invalidated(
        &self,
        member: &str,
        reader: &impl Fn(Option<CallableId>) -> Option<u32>,
    ) -> bool {
        [self.members.get(member), self.dynamic.as_ref()]
            .into_iter()
            .flatten()
            .any(|write| write.invalidated(reader))
    }

    pub(super) fn definitely_replaced(
        &self,
        member: &str,
        offset: u32,
        context: Option<CallableId>,
        region: Option<u32>,
    ) -> Option<&identity_metadata::FreshCapture> {
        let replaced = |write: &ValueAssignedMutation| {
            write.writer == mutation_history::ValueAssignedWriter::single(context)
                && write
                    .unconditional_offset
                    .is_some_and(|write| write.get() <= offset)
        };
        let direct = self.members.get(member);
        let propagated = self
            .propagated
            .as_ref()
            .and_then(|writes| writes.aggregate().members.get(member));
        if [direct, propagated]
            .into_iter()
            .flatten()
            .any(|write| write.writer != mutation_history::ValueAssignedWriter::single(context))
        {
            return None;
        }
        let events = self.replacements.as_ref()?;
        let regional = region.and_then(|region| events.regional_offset(region, member, offset));
        let global = [direct, propagated]
            .into_iter()
            .flatten()
            .any(replaced)
            .then(|| events.replacement_offset(member, offset))
            .flatten();
        let candidate = global.into_iter().chain(regional).max()?;
        if [
            self.dynamic.as_ref(),
            self.propagated
                .as_ref()
                .and_then(|writes| writes.aggregate().dynamic.as_ref()),
        ]
        .into_iter()
        .flatten()
        .any(|write| write.writer != mutation_history::ValueAssignedWriter::single(context))
        {
            return None;
        }
        events.identity.as_ref()?.captures.get(&candidate)?.as_ref()
    }
}
