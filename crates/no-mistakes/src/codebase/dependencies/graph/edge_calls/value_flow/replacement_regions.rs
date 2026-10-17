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
        self.segment_at(owner, offset).map(|(_, _, region)| region)
    }
    fn segment_at(&self, owner: Option<CallableId>, offset: u32) -> Option<(u32, u32, usize)> {
        let segments = self.owners.get(&owner)?;
        let index = segments
            .partition_point(|(start, _, _)| *start <= offset)
            .checked_sub(1)?;
        let segment = segments[index];
        (offset < segment.1).then_some(segment)
    }
}

// Only this prepare invocation owns the cursor; no graph/index layout changes.
#[derive(Default)]
pub(in super::super) struct RegionCursor {
    last: Option<(Option<CallableId>, u32, u32, usize)>,
    #[cfg(feature = "test-instrumentation")]
    searches: usize,
}
impl RegionCursor {
    pub(super) fn at(
        &mut self,
        segments: &RegionSegments,
        owner: Option<CallableId>,
        offset: u32,
    ) -> Option<usize> {
        if let Some((previous, start, end, region)) = self.last {
            if previous == owner && start <= offset && offset < end {
                return Some(region);
            }
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.searches += 1;
        }
        let segment = segments.segment_at(owner, offset);
        self.last = segment.map(|(start, end, region)| (owner, start, end, region));
        segment.map(|(_, _, region)| region)
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn searches(&self) -> usize {
        self.searches
    }
}
#[cfg(test)]
#[path = "replacement_regions/cursor_tests.rs"]
mod cursor_tests;
