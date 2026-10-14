use super::*;
use crate::codebase::dependencies::extract::ValueReplacementFacts;

#[path = "replacement_regions.rs"]
mod regions;

/// Receiver IDs come from the request's canonical resolved import identities.
/// Source spellings only select the corresponding prepared identity once.
pub(super) struct PreparedReplacements {
    pub(super) facts: std::sync::Arc<ValueReplacementFacts>,
    groups: FxHashMap<(usize, Option<CallableId>, Option<usize>), PropertyWrites>,
    regions: regions::RegionSegments,
    repeat_writes: FxHashMap<usize, (FxHashSet<String>, bool)>,
    resets: FxHashSet<usize>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) prefix_searches: ReplacementLookupProbe,
}
#[derive(Default)]
struct PropertyWrites {
    exact: FxHashMap<String, Vec<usize>>,
    dynamic: Vec<usize>,
}
#[derive(Clone, Copy)]
pub(super) struct WritePhases {
    pub(super) prefix: Option<usize>,
}
impl PreparedReplacements {
    pub(super) fn new(
        facts: std::sync::Arc<ValueReplacementFacts>,
        receivers: &FxHashMap<String, usize>,
    ) -> Self {
        let regions = regions::RegionSegments::new(&facts.regions);
        let mut result = Self {
            facts,
            groups: fx_map(),
            regions,
            repeat_writes: fx_map(),
            resets: fx_set(),
            #[cfg(feature = "test-instrumentation")]
            prefix_searches: Default::default(),
        };
        for (index, write) in result.facts.writes.iter().enumerate() {
            let Some(receiver) = receivers.get(&write.receiver) else {
                continue;
            };
            if write.receiver_reset {
                result.resets.insert(*receiver);
            }
            if write.owner.is_some()
                || write
                    .region
                    .is_some_and(|id| result.facts.regions[id].may_repeat)
            {
                let (properties, dynamic) = result.repeat_writes.entry(*receiver).or_default();
                if let Some(property) = &write.property {
                    properties.insert(property.clone());
                } else {
                    *dynamic = true;
                }
            }
            let group = result
                .groups
                .entry((*receiver, write.owner, write.region))
                .or_default();
            match &write.property {
                Some(property) => {
                    if !group.exact.contains_key(property) {
                        group.exact.insert(property.clone(), Vec::new());
                    }
                    group
                        .exact
                        .get_mut(property)
                        .expect("property index")
                        .push(index);
                }
                None => group.dynamic.push(index),
            }
        }
        for group in result.groups.values_mut() {
            for writes in group
                .exact
                .values_mut()
                .chain(std::iter::once(&mut group.dynamic))
            {
                writes.sort_unstable_by_key(|index| result.facts.writes[*index].completed_at);
            }
        }
        result
    }
    pub(super) fn properties(
        &self,
    ) -> impl Iterator<Item = (usize, Option<CallableId>, Option<usize>, &str, usize)> {
        self.groups
            .iter()
            .flat_map(|((receiver, owner, region), group)| {
                group.exact.iter().map(move |(property, writes)| {
                    (*receiver, *owner, *region, property.as_str(), writes[0])
                })
            })
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn retained_counts(&self) -> (usize, usize) {
        (
            self.groups
                .values()
                .map(|group| {
                    group.dynamic.len() + group.exact.values().map(Vec::len).sum::<usize>()
                })
                .sum(),
            self.regions.segment_count(),
        )
    }
    pub(super) fn tail(
        &self,
        receiver: usize,
        property: &str,
        owner: Option<CallableId>,
        region: Option<usize>,
    ) -> Option<usize> {
        let group = self.groups.get(&(receiver, owner, region))?;
        group
            .exact
            .get(property)
            .and_then(|writes| writes.last())
            .into_iter()
            .chain(group.dynamic.last())
            .copied()
            .max_by_key(|index| self.facts.writes[*index].completed_at)
    }
    pub(super) fn reset(&self, receiver: usize) -> bool {
        self.resets.contains(&receiver)
    }
    pub(super) fn repeat_mutated(&self, receiver: usize, property: &str) -> bool {
        self.repeat_writes
            .get(&receiver)
            .is_some_and(|(properties, dynamic)| *dynamic || properties.contains(property))
    }
    pub(super) fn region_at(&self, owner: Option<CallableId>, offset: u32) -> Option<usize> {
        self.regions.at(owner, offset)
    }
    pub(super) fn phases(
        &self,
        receiver: usize,
        property: &str,
        owner: Option<CallableId>,
        region: Option<usize>,
        offset: u32,
    ) -> Option<WritePhases> {
        let group = self.groups.get(&(receiver, owner, region))?;
        let exact = group.exact.get(property).map_or(&[][..], Vec::as_slice);
        let dynamic = &group.dynamic;
        if exact.is_empty() && dynamic.is_empty() {
            return None;
        }
        #[cfg(feature = "test-instrumentation")]
        self.prefix_searches.record(2);
        let prefix = |writes: &[usize]| {
            writes
                .get(
                    writes
                        .partition_point(|index| self.facts.writes[*index].completed_at <= offset)
                        .checked_sub(1)?,
                )
                .copied()
        };
        let latest = |left: Option<usize>, right: Option<usize>| {
            left.into_iter()
                .chain(right)
                .max_by_key(|index| self.facts.writes[*index].completed_at)
        };
        Some(WritePhases {
            prefix: latest(prefix(exact), prefix(dynamic)),
        })
    }
}
