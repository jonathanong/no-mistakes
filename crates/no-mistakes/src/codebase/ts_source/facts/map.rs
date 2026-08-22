use super::{TsFactMap, TsFactPlan, TsFileFacts};
use crate::fx::{fx_map, FxHashMap};
use std::path::PathBuf;

impl TsFactMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn with_plan(facts: FxHashMap<PathBuf, TsFileFacts>, plan: TsFactPlan) -> Self {
        Self {
            owned: facts,
            shared: fx_map(),
            plan,
        }
    }

    pub(crate) fn from_iter_with_plan(
        facts: impl IntoIterator<Item = (PathBuf, TsFileFacts)>,
        plan: TsFactPlan,
    ) -> Self {
        Self::with_plan(facts.into_iter().collect(), plan)
    }

    pub(crate) fn from_shared_iter_with_plan(
        facts: impl IntoIterator<Item = (PathBuf, std::sync::Arc<TsFileFacts>)>,
        plan: TsFactPlan,
    ) -> Self {
        Self {
            owned: fx_map(),
            shared: facts.into_iter().collect(),
            plan,
        }
    }

    pub(crate) fn plan(&self) -> TsFactPlan {
        self.plan
    }

    pub fn get(&self, path: &std::path::Path) -> Option<&TsFileFacts> {
        if self.shared.is_empty() {
            return self.owned.get(path);
        }
        if self.owned.is_empty() {
            return self.shared.get(path).map(std::sync::Arc::as_ref);
        }
        self.owned
            .get(path)
            .or_else(|| self.shared.get(path).map(std::sync::Arc::as_ref))
    }

    pub fn get_mut(&mut self, path: &std::path::Path) -> Option<&mut TsFileFacts> {
        if !self.owned.contains_key(path) {
            let facts = self
                .shared
                .remove(path)
                .map(std::sync::Arc::unwrap_or_clone)?;
            self.owned.insert(path.to_path_buf(), facts);
        }
        self.owned.get_mut(path)
    }

    pub fn insert(&mut self, path: PathBuf, facts: TsFileFacts) -> Option<TsFileFacts> {
        let shared = self
            .shared
            .remove(&path)
            .map(std::sync::Arc::unwrap_or_clone);
        self.owned.insert(path, facts).or(shared)
    }

    pub fn remove(&mut self, path: &std::path::Path) -> Option<TsFileFacts> {
        self.owned.remove(path).or_else(|| {
            self.shared
                .remove(path)
                .map(std::sync::Arc::unwrap_or_clone)
        })
    }

    pub fn contains_key(&self, path: &std::path::Path) -> bool {
        self.owned.contains_key(path) || self.shared.contains_key(path)
    }

    pub fn is_empty(&self) -> bool {
        self.owned.is_empty() && self.shared.is_empty()
    }

    pub fn len(&self) -> usize {
        self.owned.len() + self.shared.len()
    }

    pub fn keys(&self) -> impl Iterator<Item = &PathBuf> {
        self.owned.keys().chain(self.shared.keys())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &TsFileFacts)> {
        self.owned
            .iter()
            .chain(self.shared.iter().map(as_ref_entry))
    }

    pub fn values(&self) -> impl Iterator<Item = &TsFileFacts> {
        self.owned
            .values()
            .chain(self.shared.values().map(std::sync::Arc::as_ref))
    }

    pub(crate) fn extend(&mut self, other: Self) {
        for (path, facts) in other.owned {
            self.shared.remove(&path);
            self.owned.insert(path, facts);
        }
        for (path, facts) in other.shared {
            self.owned.remove(&path);
            self.shared.insert(path, facts);
        }
    }

    fn materialize_shared(&mut self) {
        self.owned.extend(
            std::mem::take(&mut self.shared)
                .into_iter()
                .map(unwrap_entry),
        );
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
        Self::with_plan(entries.into_iter().collect(), TsFactPlan::default())
    }
}

impl IntoIterator for TsFactMap {
    type Item = (PathBuf, TsFileFacts);
    type IntoIter = std::iter::Chain<
        std::collections::hash_map::IntoIter<PathBuf, TsFileFacts>,
        std::iter::Map<
            std::collections::hash_map::IntoIter<PathBuf, std::sync::Arc<TsFileFacts>>,
            fn((PathBuf, std::sync::Arc<TsFileFacts>)) -> (PathBuf, TsFileFacts),
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.owned.into_iter().chain(self.shared.into_iter().map(
            unwrap_entry as fn((PathBuf, std::sync::Arc<TsFileFacts>)) -> (PathBuf, TsFileFacts),
        ))
    }
}

impl<'a> IntoIterator for &'a TsFactMap {
    type Item = (&'a PathBuf, &'a TsFileFacts);
    type IntoIter = std::iter::Chain<
        std::collections::hash_map::Iter<'a, PathBuf, TsFileFacts>,
        std::iter::Map<
            std::collections::hash_map::Iter<'a, PathBuf, std::sync::Arc<TsFileFacts>>,
            fn((&'a PathBuf, &'a std::sync::Arc<TsFileFacts>)) -> (&'a PathBuf, &'a TsFileFacts),
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.owned.iter().chain(self.shared.iter().map(
            as_ref_entry
                as fn(
                    (&'a PathBuf, &'a std::sync::Arc<TsFileFacts>),
                ) -> (&'a PathBuf, &'a TsFileFacts),
        ))
    }
}

impl<'a> IntoIterator for &'a mut TsFactMap {
    type Item = (&'a PathBuf, &'a mut TsFileFacts);
    type IntoIter = std::collections::hash_map::IterMut<'a, PathBuf, TsFileFacts>;

    fn into_iter(self) -> Self::IntoIter {
        self.materialize_shared();
        self.owned.iter_mut()
    }
}

fn unwrap_entry((path, facts): (PathBuf, std::sync::Arc<TsFileFacts>)) -> (PathBuf, TsFileFacts) {
    (path, std::sync::Arc::unwrap_or_clone(facts))
}

fn as_ref_entry<'a>(
    (path, facts): (&'a PathBuf, &'a std::sync::Arc<TsFileFacts>),
) -> (&'a PathBuf, &'a TsFileFacts) {
    (path, facts.as_ref())
}
