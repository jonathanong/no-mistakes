use super::*;
use mutation_history::ValueAssignedWriter;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshCapture {
    pub origin: nested::ReceiverRead,
    pub reads: capture_reads::CaptureReads,
    pub epoch: u64,
    pub phase: Option<(u32, u32)>,
    pub completed_at: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdentityEvents {
    offsets: BTreeSet<u32>,
    writer: Option<ValueAssignedWriter>,
}
impl IdentityEvents {
    pub(super) fn record(&mut self, offset: u32, writer: ValueAssignedWriter) {
        self.offsets.insert(offset);
        self.writer = Some(self.writer.map_or(writer, |old| {
            if old == writer {
                old
            } else {
                ValueAssignedWriter::Mixed
            }
        }));
    }
    fn merge(&mut self, source: Self) {
        if let Some(writer) = source.writer {
            self.writer = Some(self.writer.map_or(writer, |old| {
                if old == writer {
                    old
                } else {
                    ValueAssignedWriter::Mixed
                }
            }));
        }
        self.offsets.extend(source.offsets);
    }
    pub(super) fn changed(&self, left: nested::ReceiverRead, right: nested::ReceiverRead) -> bool {
        let (Some((left, context)), Some((right, other))) = (left, right) else {
            return true;
        };
        if left == right && context == other {
            return false;
        }
        if self.offsets.is_empty() {
            return false;
        }
        if context != other || self.writer != Some(ValueAssignedWriter::single(context)) {
            return true;
        }
        between(&self.offsets, left, right)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdentityMetadata {
    pub captures: FxHashMap<u32, Option<FreshCapture>>,
    preserved: FxHashMap<String, BTreeSet<u32>>,
    dynamic: BTreeSet<u32>,
    pub unknown: IdentityEvents,
}
impl IdentityMetadata {
    pub(super) fn preserve(&mut self, member: Option<&str>, offset: u32) {
        match member {
            Some(member) => {
                if !self.preserved.contains_key(member) {
                    self.preserved.insert(member.to_string(), BTreeSet::new());
                }
                self.preserved
                    .get_mut(member)
                    .expect("inserted member")
                    .insert(offset);
            }
            None => {
                self.dynamic.insert(offset);
            }
        }
    }
    pub(super) fn capture(&mut self, offset: u32, capture: FreshCapture) {
        use std::collections::hash_map::Entry;
        match self.captures.entry(offset) {
            Entry::Vacant(entry) => {
                entry.insert(Some(capture));
            }
            Entry::Occupied(mut entry) => {
                if entry.get().as_ref().is_some_and(|old| old != &capture) {
                    entry.insert(None);
                }
            }
        }
    }
    pub(super) fn merge(&mut self, source: Self) {
        for (offset, capture) in source.captures {
            use std::collections::hash_map::Entry;
            match self.captures.entry(offset) {
                Entry::Vacant(entry) => {
                    entry.insert(capture);
                }
                Entry::Occupied(mut entry) => {
                    if entry.get() != &capture {
                        entry.insert(None);
                    }
                }
            }
        }
        for (member, events) in source.preserved {
            self.preserved.entry(member).or_default().extend(events);
        }
        self.dynamic.extend(source.dynamic);
        self.unknown.merge(source.unknown);
    }
    pub(super) fn changed(&self, member: &str, left: u32, right: u32) -> bool {
        self.preserved
            .get(member)
            .is_some_and(|events| between(events, left, right))
            || between(&self.dynamic, left, right)
    }
}
pub(super) fn between(events: &BTreeSet<u32>, left: u32, right: u32) -> bool {
    left != right
        && events
            .range((
                std::ops::Bound::Excluded(left.min(right)),
                std::ops::Bound::Included(left.max(right)),
            ))
            .next()
            .is_some()
}
pub(super) fn map_between<T>(events: &BTreeMap<u32, T>, left: u32, right: u32) -> bool {
    left != right
        && events
            .range((
                std::ops::Bound::Excluded(left.min(right)),
                std::ops::Bound::Included(left.max(right)),
            ))
            .next()
            .is_some()
}
