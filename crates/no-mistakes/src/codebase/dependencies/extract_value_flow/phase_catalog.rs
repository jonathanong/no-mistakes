use super::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PhaseCatalog {
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) segments: Vec<(u32, u32, u32)>,
}
impl PhaseCatalog {
    fn phase(&self, offset: u32) -> Option<u32> {
        if offset >= self.end {
            return None;
        }
        let end = self.segments.partition_point(|segment| segment.0 <= offset);
        (end > 0)
            .then(|| self.segments[end - 1])
            .filter(|segment| offset < segment.1)
            .map(|segment| segment.2)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct PhaseRead {
    catalog: Arc<PhaseCatalog>,
    first_write: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ReadPhases {
    pending: FxHashMap<u32, PhaseRead>,
    prepared: Vec<PhaseRead>,
}
impl ReadPhases {
    pub(super) fn record(&mut self, catalog: &Arc<PhaseCatalog>, phase: u32) {
        let read = self
            .pending
            .entry(catalog.start)
            .or_insert_with(|| PhaseRead {
                catalog: Arc::clone(catalog),
                first_write: phase,
            });
        read.first_write = read.first_write.min(phase);
    }
    pub(super) fn merge(&mut self, source: Self) {
        for (_, read) in source.pending {
            self.record(&read.catalog, read.first_write);
        }
    }
    pub(super) fn finish(&mut self) {
        self.prepared = std::mem::take(&mut self.pending).into_values().collect();
        self.prepared
            .sort_unstable_by_key(|read| read.catalog.start);
    }
    pub(super) fn invalidated(&self, offset: u32) -> bool {
        let end = self
            .prepared
            .partition_point(|read| read.catalog.start <= offset);
        end > 0
            && self.prepared[end - 1]
                .catalog
                .phase(offset)
                .is_some_and(|phase| self.prepared[end - 1].first_write <= phase)
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn len(&self) -> usize {
        self.prepared.len()
    }
}
