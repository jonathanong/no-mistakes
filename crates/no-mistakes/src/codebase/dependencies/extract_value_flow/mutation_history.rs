use super::*;

/// One prefix summary per mutation receiver path, shared by every default that
/// refers to that receiver. Neither finalization nor lookup replays all writes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueMutationHistory {
    pub summary: Option<ValueMutationInfo>,
    pub members: FxHashMap<String, Self>,
    pub dynamic: Option<Box<Self>>,
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
        let (member, receiver, projection) = if let Some((head, tail)) = receiver.split_first() {
            (head.as_deref(), tail, projection)
        } else if let Some((head, tail)) = projection.split_first() {
            (Some(head.as_str()), receiver, tail)
        } else {
            return self.summary.as_ref().is_some_and(|mutation| {
                external
                    || mutation.offset <= offset
                    || mutation.contexts.iter().any(|writer| *writer != context)
            });
        };
        let matches =
            |child: &Self| child.invalidated(receiver, projection, offset, context, external);
        if self.dynamic.as_deref().is_some_and(matches) {
            return true;
        }
        match member {
            Some(member) => self.members.get(member).is_some_and(matches),
            None => self.members.values().any(matches),
        }
    }
}
