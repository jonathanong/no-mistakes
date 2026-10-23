use crate::fx::FxHashMap;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Shared table for alternative snapshots. Cloning is pointer-sized until a write.
#[derive(Clone)]
pub(in crate::codebase::postgres::query_annotation) struct SnapshotMap<K, V>(Arc<FxHashMap<K, V>>);

impl<K, V> Default for SnapshotMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self(Arc::new(FxHashMap::default()))
    }
}

impl<K, V> PartialEq for SnapshotMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K, V> Eq for SnapshotMap<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
}

impl<K, V> Debug for SnapshotMap<K, V>
where
    K: Debug + Eq + Hash,
    V: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<K, V> SnapshotMap<K, V>
where
    K: Eq + Hash,
{
    pub(super) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<K, V> SnapshotMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub(super) fn into_map(self) -> FxHashMap<K, V> {
        match Arc::try_unwrap(self.0) {
            Ok(entries) => entries,
            Err(entries) => entries.as_ref().clone(),
        }
    }
}

impl<K, V> Deref for SnapshotMap<K, V> {
    type Target = FxHashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for SnapshotMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

impl<'a, K, V> IntoIterator for &'a SnapshotMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut SnapshotMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = std::collections::hash_map::IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Arc::make_mut(&mut self.0).iter_mut()
    }
}

impl<K, V> From<FxHashMap<K, V>> for SnapshotMap<K, V>
where
    K: Eq + Hash,
{
    fn from(entries: FxHashMap<K, V>) -> Self {
        Self(Arc::new(entries))
    }
}

impl<K, V> FromIterator<(K, V)> for SnapshotMap<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(entries: T) -> Self {
        Self(Arc::new(entries.into_iter().collect()))
    }
}
