use super::*;
use crate::codebase::dependencies::extract::ValueRepeatRegion;

/// Linear retained segments resume an enclosing region after an inner loop.
/// No region retains copies of all enclosing intervals or write prefixes.
pub(super) struct RegionSegments {
    owners: FxHashMap<Option<CallableId>, Vec<(u32, u32, usize)>>,
}
impl RegionSegments {
    pub(super) fn new(regions: &[ValueRepeatRegion]) -> Self {
        let mut events: FxHashMap<_, Vec<_>> = fx_map();
        for (id, region) in regions.iter().enumerate() {
            events
                .entry(region.owner)
                .or_default()
                .push((region.start, true, id));
            events
                .entry(region.owner)
                .or_default()
                .push((region.end, false, id));
        }
        let mut owners = fx_map();
        for (owner, mut events) in events {
            events.sort_unstable();
            let mut active = std::collections::BTreeSet::new();
            let mut segments = Vec::new();
            let mut previous = 0;
            for (offset, start, id) in events {
                if offset > previous {
                    if let Some((_, region)) = active.last() {
                        segments.push((previous, offset, *region));
                    }
                }
                if start {
                    active.insert((regions[id].start, id));
                } else {
                    active.remove(&(regions[id].start, id));
                }
                previous = offset;
            }
            owners.insert(owner, segments);
        }
        Self { owners }
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn segment_count(&self) -> usize {
        self.owners.values().map(Vec::len).sum()
    }
    pub(super) fn at(&self, owner: Option<CallableId>, offset: u32) -> Option<usize> {
        let segments = self.owners.get(&owner)?;
        let index = segments
            .partition_point(|(start, _, _)| *start <= offset)
            .checked_sub(1)?;
        let (_, end, region) = segments[index];
        (offset < end).then_some(region)
    }
}
