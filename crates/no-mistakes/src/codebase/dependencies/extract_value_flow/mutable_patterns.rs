use super::*;
use std::collections::BTreeMap;
use std::sync::Arc;

struct RootPhases {
    catalog: Arc<phase_catalog::PhaseCatalog>,
    initial: Option<usize>,
    versions: BTreeMap<u32, usize>,
    retains_initial: bool,
}
#[derive(Default)]
pub(super) struct PatternRoots {
    writes: FxHashMap<Option<CallableId>, BTreeMap<u32, RootPhases>>,
}
impl PatternRoots {
    pub(super) fn previous(
        &self,
        catalog: &phase_catalog::PhaseCatalog,
        phase: u32,
        owner: Option<CallableId>,
    ) -> Option<usize> {
        let entry = self.writes.get(&owner)?.get(&catalog.start)?;
        entry
            .versions
            .range(..phase)
            .next_back()
            .map(|(_, id)| *id)
            .or(entry.initial)
    }
    pub(super) fn record(
        &mut self,
        catalog: &Arc<phase_catalog::PhaseCatalog>,
        phase: u32,
        owner: Option<CallableId>,
        initial: Option<usize>,
        selected: usize,
        guaranteed: bool,
    ) {
        let writes = self.writes.entry(owner).or_default();
        let entry = writes.entry(catalog.start).or_insert_with(|| RootPhases {
            catalog: Arc::clone(catalog),
            initial,
            versions: BTreeMap::new(),
            retains_initial: false,
        });
        entry.retains_initial = !guaranteed
            && if entry.versions.is_empty() {
                initial.is_some()
            } else {
                entry.retains_initial
            };
        entry.versions.insert(phase, selected);
    }
    pub(super) fn final_selected(
        &self,
        start: u32,
        owner: Option<CallableId>,
    ) -> Option<(usize, Option<usize>, bool)> {
        let entry = self.writes.get(&owner)?.get(&start)?;
        Some((
            *entry.versions.last_key_value()?.1,
            entry.initial,
            entry.retains_initial,
        ))
    }
    pub(super) fn selected(&self, read: nested::RootRead) -> Option<Option<usize>> {
        let (_, entry) = self.writes.get(&read.1)?.range(..=read.0).next_back()?;
        let phase = entry.catalog.phase(read.0)?;
        Some(
            entry
                .versions
                .range(..=phase)
                .next_back()
                .map(|(_, id)| *id)
                .or(entry.initial),
        )
    }
}
