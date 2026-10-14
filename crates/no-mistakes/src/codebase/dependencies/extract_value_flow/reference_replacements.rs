use super::*;
use std::collections::BTreeMap;

/// Only prepared nested-reference components retain these shared timelines.
/// Conditional fresh writes preserve an established identity without proving it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceReplacements {
    pub members: FxHashMap<String, BTreeMap<u32, bool>>,
    pub dynamic: BTreeMap<u32, bool>,
}
impl ReferenceReplacements {
    pub(super) fn record(
        &mut self,
        member: Option<&str>,
        offset: u32,
        fresh: bool,
        guaranteed: bool,
    ) {
        if fresh && (!guaranteed || member.is_none()) {
            return;
        }
        let events = match member {
            Some(member) => {
                if !self.members.contains_key(member) {
                    self.members.insert(member.to_string(), BTreeMap::new());
                }
                self.members
                    .get_mut(member)
                    .expect("inserted member timeline")
            }
            None => &mut self.dynamic,
        };
        let distinct = fresh && guaranteed;
        // Duplicate visits can only weaken certainty at the same completion.
        events
            .entry(offset)
            .and_modify(|value| *value &= distinct)
            .or_insert(distinct);
    }
    pub(super) fn merge(&mut self, source: Self) {
        for (member, events) in source.members {
            merge_events(self.members.entry(member).or_default(), events);
        }
        merge_events(&mut self.dynamic, source.dynamic);
    }
    pub(super) fn replaced(&self, member: &str, offset: u32) -> bool {
        let direct = self
            .members
            .get(member)
            .and_then(|events| events.range(..=offset).next_back());
        let dynamic = self.dynamic.range(..=offset).next_back();
        match (direct, dynamic) {
            (Some((direct_offset, fresh)), Some((dynamic_offset, _))) => {
                direct_offset > dynamic_offset && *fresh
            }
            (Some((_, fresh)), None) => *fresh,
            _ => false,
        }
    }
}
fn merge_events(target: &mut BTreeMap<u32, bool>, source: BTreeMap<u32, bool>) {
    for (offset, fresh) in source {
        target
            .entry(offset)
            .and_modify(|value| *value &= fresh)
            .or_insert(fresh);
    }
}

#[cfg(test)]
#[path = "reference_replacements_tests.rs"]
mod tests;
