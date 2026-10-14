use super::*;

/// One prefix summary per mutation receiver path, shared by every default that
/// refers to that receiver. Neither finalization nor lookup replays all writes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueMutationHistory {
    pub summary: Option<ValueMutationInfo>,
    pub assigned: Option<Box<ValueAssignedMutations>>,
    pub has_assignments: bool,
    pub members: FxHashMap<String, Self>,
    pub dynamic: Option<Box<Self>>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueAssignedMutations {
    pub members: FxHashMap<String, ValueAssignedMutation>,
    pub dynamic: Option<ValueAssignedMutation>,
    #[cfg(feature = "test-instrumentation")]
    pub merge_reinsertions: usize,
}
/// Exact writes need only one context, or the fact that writers differ.
/// With two distinct writers, at least one differs from every reader.
/// Copy keeps each property summary inline instead of allocating a context set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueAssignedMutation {
    pub offset: u32,
    pub context: Option<CallableId>,
    pub multiple_contexts: bool,
}
impl ValueMutationHistory {
    pub(super) fn record(
        &mut self,
        path: &[Option<String>],
        offset: u32,
        context: Option<CallableId>,
    ) {
        let mut node = self;
        for member in path.iter().map(Some).chain(std::iter::once(None)) {
            let summary = node.summary.get_or_insert_with(|| ValueMutationInfo {
                offset,
                contexts: fx_set(),
            });
            summary.offset = summary.offset.min(offset);
            summary.contexts.insert(context);
            if let Some(member) = member {
                node = match member {
                    Some(member) => node.members.entry(member.clone()).or_default(),
                    None => node.dynamic.get_or_insert_with(Default::default),
                };
            }
        }
    }
    pub(super) fn merge(&mut self, source: Self) {
        self.has_assignments |= source.has_assignments;
        if let Some(assigned) = source.assigned {
            if let Some(target) = &mut self.assigned {
                target.merge(*assigned);
            } else {
                self.assigned = Some(assigned);
            }
        }
        if let Some(source) = source.summary {
            let target = self.summary.get_or_insert_with(|| ValueMutationInfo {
                offset: source.offset,
                contexts: fx_set(),
            });
            target.offset = target.offset.min(source.offset);
            target.contexts.extend(source.contexts);
        }
        for (member, child) in source.members {
            match self.members.entry(member) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().merge(child)
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(child);
                }
            }
        }
        if let Some(child) = source.dynamic {
            if let Some(target) = &mut self.dynamic {
                target.merge(*child);
            } else {
                self.dynamic = Some(child);
            }
        }
    }
    pub fn invalidated(
        &self,
        receiver: &[Option<String>],
        projection: &[String],
        offset: u32,
        context: Option<CallableId>,
        external: bool,
    ) -> bool {
        self.invalidated_path(
            receiver
                .iter()
                .map(Option::as_deref)
                .chain(projection.iter().map(|member| Some(member.as_str()))),
            offset,
            context,
            external,
        )
    }
    pub fn invalidated_projected(
        &self,
        receiver: &[Option<String>],
        projection: &[&str],
        offset: u32,
        context: Option<CallableId>,
        external: bool,
    ) -> bool {
        self.invalidated_path(
            receiver
                .iter()
                .map(Option::as_deref)
                .chain(projection.iter().rev().map(|member| Some(*member))),
            offset,
            context,
            external,
        )
    }
    fn invalidated_path<'a>(
        &self,
        mut path: impl Iterator<Item = Option<&'a str>> + Clone,
        offset: u32,
        context: Option<CallableId>,
        external: bool,
    ) -> bool {
        let Some(member) = path.next() else {
            return self.summary.as_ref().is_some_and(|mutation| {
                external
                    || mutation.offset <= offset
                    || mutation.contexts.iter().any(|writer| *writer != context)
            });
        };
        let matches =
            |child: &Self| child.invalidated_path(path.clone(), offset, context, external);
        if self.dynamic.as_deref().is_some_and(matches) {
            return true;
        }
        match member {
            Some(member) => self.members.get(member).is_some_and(matches),
            None => self.members.values().any(matches),
        }
    }
}
