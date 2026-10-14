use super::*;
use std::collections::BTreeMap;

type RegionalReplacementEvents = FxHashMap<u32, FxHashMap<String, std::collections::BTreeSet<u32>>>;

/// Only prepared nested-reference components retain these shared timelines.
/// Conditional fresh writes preserve an established identity without proving it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceReplacements {
    #[cfg(feature = "test-instrumentation")]
    pub region_record_attempts: usize,
    #[cfg(feature = "test-instrumentation")]
    pub region_queries: RegionQueryCount,
    pub members: FxHashMap<String, BTreeMap<u32, bool>>,
    pub dynamic: BTreeMap<u32, bool>,
    pub regions: Option<Box<RegionalReplacementEvents>>,
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
        #[cfg(feature = "test-instrumentation")]
        {
            self.region_record_attempts += source.region_record_attempts;
            self.region_queries
                .set(self.region_queries.get() + source.region_queries.get());
        }
        if let Some(regions) = source.regions {
            let target = self.regions.get_or_insert_with(Default::default);
            for (region, members) in *regions {
                for (member, events) in members {
                    target
                        .entry(region)
                        .or_default()
                        .entry(member)
                        .or_default()
                        .extend(events);
                }
            }
        }
        for (member, events) in source.members {
            merge_events(self.members.entry(member).or_default(), events);
        }
        merge_events(&mut self.dynamic, source.dynamic);
    }
    pub(super) fn record_region(&mut self, region: u32, member: &str, offset: u32) {
        #[cfg(feature = "test-instrumentation")]
        {
            self.region_record_attempts += 1;
        }
        let members = self
            .regions
            .get_or_insert_with(Default::default)
            .entry(region)
            .or_default();
        if !members.contains_key(member) {
            members.insert(member.to_string(), Default::default());
        }
        members
            .get_mut(member)
            .expect("inserted regional member")
            .insert(offset);
    }
    pub(super) fn replaced_in_region(&self, region: u32, member: &str, offset: u32) -> bool {
        #[cfg(feature = "test-instrumentation")]
        {
            self.region_queries
                .0
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let Some(fresh) = self
            .regions
            .as_ref()
            .and_then(|regions| regions.get(&region))
            .and_then(|members| members.get(member))
            .and_then(|events| events.range(..=offset).next_back())
        else {
            return false;
        };
        // Every possible-original restoration remains in the global timeline,
        // even when written in another region. Future writes cannot alter snapshots.
        let original = self
            .members
            .get(member)
            .and_then(|events| events.range(..=offset).next_back())
            .filter(|(_, fresh)| !**fresh)
            .map(|(offset, _)| offset);
        let dynamic = self
            .dynamic
            .range(..=offset)
            .next_back()
            .map(|(offset, _)| offset);
        original
            .into_iter()
            .chain(dynamic)
            .all(|offset| offset < fresh)
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

#[cfg(feature = "test-instrumentation")]
#[derive(Debug, Default)]
pub struct RegionQueryCount(std::sync::atomic::AtomicUsize);
#[cfg(feature = "test-instrumentation")]
impl RegionQueryCount {
    pub fn get(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn set(&self, count: usize) {
        self.0.store(count, std::sync::atomic::Ordering::Relaxed);
    }
}
#[cfg(feature = "test-instrumentation")]
impl Clone for RegionQueryCount {
    fn clone(&self) -> Self {
        Self(std::sync::atomic::AtomicUsize::new(self.get()))
    }
}
#[cfg(feature = "test-instrumentation")]
impl PartialEq for RegionQueryCount {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}
#[cfg(feature = "test-instrumentation")]
impl Eq for RegionQueryCount {}
