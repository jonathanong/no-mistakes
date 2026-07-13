use super::{TsFactMap, TsFactPlan, TsFileFacts};
use std::collections::HashMap;
use std::path::PathBuf;

impl TsFactMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn with_plan(facts: HashMap<PathBuf, TsFileFacts>, plan: TsFactPlan) -> Self {
        Self { facts, plan }
    }

    pub(crate) fn plan(&self) -> TsFactPlan {
        self.plan
    }
}

impl std::ops::Deref for TsFactMap {
    type Target = HashMap<PathBuf, TsFileFacts>;

    fn deref(&self) -> &Self::Target {
        &self.facts
    }
}

impl std::ops::DerefMut for TsFactMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.facts
    }
}

impl<const N: usize> From<[(PathBuf, TsFileFacts); N]> for TsFactMap {
    fn from(entries: [(PathBuf, TsFileFacts); N]) -> Self {
        Self::with_plan(HashMap::from(entries), TsFactPlan::default())
    }
}

impl IntoIterator for TsFactMap {
    type Item = (PathBuf, TsFileFacts);
    type IntoIter = std::collections::hash_map::IntoIter<PathBuf, TsFileFacts>;

    fn into_iter(self) -> Self::IntoIter {
        self.facts.into_iter()
    }
}

impl<'a> IntoIterator for &'a TsFactMap {
    type Item = (&'a PathBuf, &'a TsFileFacts);
    type IntoIter = std::collections::hash_map::Iter<'a, PathBuf, TsFileFacts>;

    fn into_iter(self) -> Self::IntoIter {
        self.facts.iter()
    }
}

impl<'a> IntoIterator for &'a mut TsFactMap {
    type Item = (&'a PathBuf, &'a mut TsFileFacts);
    type IntoIter = std::collections::hash_map::IterMut<'a, PathBuf, TsFileFacts>;

    fn into_iter(self) -> Self::IntoIter {
        self.facts.iter_mut()
    }
}
