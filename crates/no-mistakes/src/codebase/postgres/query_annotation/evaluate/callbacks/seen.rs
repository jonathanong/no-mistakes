//! Branch snapshots share callback identities until a path changes them.
use super::CallbackIdentity;
use crate::fx::FxHashSet;
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Debug, Default)]
pub(in crate::codebase::postgres::query_annotation) struct CallbackSet {
    keys: Arc<FxHashSet<CallbackIdentity>>,
    // An upper bound remains valid after removal, avoiding a second key scan.
    max_frame: Option<usize>,
}

impl PartialEq for CallbackSet {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys
    }
}
impl Eq for CallbackSet {}

impl Deref for CallbackSet {
    type Target = FxHashSet<CallbackIdentity>;
    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl CallbackSet {
    pub fn insert(&mut self, key: CallbackIdentity) -> bool {
        if Arc::strong_count(&self.keys) > 1 && self.keys.contains(&key) {
            return false;
        }
        self.max_frame = Some(self.max_frame.map_or(key.2, |frame| frame.max(key.2)));
        Arc::make_mut(&mut self.keys).insert(key)
    }

    pub fn remove(&mut self, key: &CallbackIdentity) -> bool {
        // Stateful revisits must detach only the path that clears this key.
        if Arc::strong_count(&self.keys) > 1 && !self.keys.contains(key) {
            return false;
        }
        Arc::make_mut(&mut self.keys).remove(key)
    }

    pub fn retain(&mut self, predicate: impl FnMut(&CallbackIdentity) -> bool) {
        Arc::make_mut(&mut self.keys).retain(predicate);
    }

    pub fn shares(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.keys, &other.keys)
    }

    pub fn below(&self, preserved: usize) -> bool {
        self.max_frame.is_none_or(|frame| frame < preserved)
    }
}

impl FromIterator<CallbackIdentity> for CallbackSet {
    fn from_iter<T: IntoIterator<Item = CallbackIdentity>>(iter: T) -> Self {
        let mut result = Self::default();
        for key in iter {
            result.insert(key);
        }
        result
    }
}

impl IntoIterator for CallbackSet {
    type Item = CallbackIdentity;
    type IntoIter = std::collections::hash_set::IntoIter<CallbackIdentity>;
    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.keys).into_iter()
    }
}
