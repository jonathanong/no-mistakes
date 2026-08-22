use super::{TsFactMap, TsFactPlan, TsFactSlot, TsFileFacts};
use crate::codebase::ts_source::FileIdMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl TsFactMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from_iter_with_plan(
        facts: impl IntoIterator<Item = (PathBuf, TsFileFacts)>,
        plan: TsFactPlan,
    ) -> Self {
        Self {
            facts: FileIdMap::from_entries(
                facts
                    .into_iter()
                    .map(|(path, facts)| (path, TsFactSlot::Owned(Box::new(facts)))),
            ),
            plan,
            ..Self::default()
        }
    }

    pub(crate) fn from_iter_with_plan_and_inventory(
        facts: impl IntoIterator<Item = (PathBuf, TsFileFacts)>,
        plan: TsFactPlan,
        inventory: Arc<crate::codebase::ts_source::FileInventory>,
    ) -> Self {
        Self {
            facts: FileIdMap::from_iter_with_inventory(
                facts
                    .into_iter()
                    .map(|(path, facts)| (path, TsFactSlot::Owned(Box::new(facts)))),
                inventory,
            ),
            plan,
            ..Self::default()
        }
    }

    pub(crate) fn from_shared_iter_with_plan(
        facts: impl IntoIterator<Item = (PathBuf, Arc<TsFileFacts>)>,
        plan: TsFactPlan,
    ) -> Self {
        Self {
            facts: FileIdMap::from_entries(
                facts
                    .into_iter()
                    .map(|(path, facts)| (path, TsFactSlot::Shared(facts))),
            ),
            plan,
            ..Self::default()
        }
    }

    pub(crate) fn plan(&self) -> TsFactPlan {
        self.plan
    }

    pub(crate) fn bump_playwright_scan_generation(&mut self) {
        self.playwright_scan_generation = self.playwright_scan_generation.wrapping_add(1);
    }

    pub(crate) fn playwright_scan_cache_key(
        &self,
        settings: &crate::playwright::config::Settings,
    ) -> (u64, crate::codebase::check_facts::PlaywrightSettingsKey) {
        (
            self.playwright_scan_generation,
            crate::codebase::check_facts::PlaywrightSettingsKey::new(settings),
        )
    }

    pub fn get(&self, path: &Path) -> Option<&TsFileFacts> {
        self.facts.get(path).map(TsFactSlot::as_facts)
    }

    pub fn get_mut(&mut self, path: &Path) -> Option<&mut TsFileFacts> {
        let slot = self.facts.get_mut(path)?;
        slot.materialize_owned();
        Some(slot.as_facts_mut())
    }

    pub fn insert(&mut self, path: PathBuf, facts: TsFileFacts) -> Option<TsFileFacts> {
        self.facts
            .insert(path, TsFactSlot::Owned(Box::new(facts)))
            .map(TsFactSlot::into_owned)
    }

    pub fn remove(&mut self, path: &Path) -> Option<TsFileFacts> {
        self.facts.remove(path).map(TsFactSlot::into_owned)
    }

    pub fn contains_key(&self, path: &Path) -> bool {
        self.facts.contains_key(path)
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn keys(&self) -> impl Iterator<Item = &PathBuf> {
        self.facts.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &TsFileFacts)> {
        self.facts
            .iter()
            .map(|(path, slot)| (path, slot.as_facts()))
    }

    pub fn values(&self) -> impl Iterator<Item = &TsFileFacts> {
        self.facts.values().map(TsFactSlot::as_facts)
    }

    pub(crate) fn extend(&mut self, other: Self) {
        if other.is_empty() {
            return;
        }
        self.facts.extend(other.facts);
        self.bump_playwright_scan_generation();
    }
}

impl std::ops::Index<&PathBuf> for TsFactMap {
    type Output = TsFileFacts;

    fn index(&self, path: &PathBuf) -> &Self::Output {
        self.get(path).expect("TS fact path is not present")
    }
}

impl<const N: usize> From<[(PathBuf, TsFileFacts); N]> for TsFactMap {
    fn from(entries: [(PathBuf, TsFileFacts); N]) -> Self {
        Self::from_iter_with_plan(entries, TsFactPlan::default())
    }
}
