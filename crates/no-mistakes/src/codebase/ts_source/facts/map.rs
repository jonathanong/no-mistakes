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
        }
    }

    pub(crate) fn plan(&self) -> TsFactPlan {
        self.plan
    }

    pub fn get(&self, path: &Path) -> Option<&TsFileFacts> {
        self.facts.get(path).map(TsFactSlot::as_facts)
    }

    pub fn get_mut(&mut self, path: &Path) -> Option<&mut TsFileFacts> {
        let slot = self.facts.get_mut(path)?;
        if matches!(slot, TsFactSlot::Shared(_)) {
            let owned = slot.clone().into_owned();
            *slot = TsFactSlot::Owned(Box::new(owned));
        }
        match slot {
            TsFactSlot::Owned(facts) => Some(facts),
            TsFactSlot::Shared(_) => None,
        }
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
        self.facts.extend(other.facts);
    }

    #[cfg(test)]
    pub(crate) fn extend_shared(
        &mut self,
        facts: impl IntoIterator<Item = (PathBuf, Arc<TsFileFacts>)>,
    ) {
        for (path, facts) in facts {
            self.facts.insert(path, TsFactSlot::Shared(facts));
        }
    }

    #[cfg(test)]
    pub(crate) fn shared_arc(&self, path: &Path) -> Option<&Arc<TsFileFacts>> {
        match self.facts.get(path)? {
            TsFactSlot::Shared(facts) => Some(facts),
            TsFactSlot::Owned(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn has_owned(&self, path: &Path) -> bool {
        matches!(self.facts.get(path), Some(TsFactSlot::Owned(_)))
    }

    #[cfg(test)]
    pub(crate) fn shared_is_empty(&self) -> bool {
        self.facts
            .iter()
            .all(|(_, slot)| matches!(slot, TsFactSlot::Owned(_)))
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
