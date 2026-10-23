use super::dense_slots::DenseSlots;
use crate::fx::FxHashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Mutable identities retain shared immutable dense snapshots between writes.
/// The map itself is copy-on-write so an alternative can snapshot every object
/// without cloning the table until an arm inserts or replaces one.
#[derive(Clone, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct ArgumentObjects(
    Arc<FxHashMap<u64, DenseSlots>>,
);

impl ArgumentObjects {
    pub(super) fn insert(&mut self, id: u64, slots: impl Into<DenseSlots>) -> Option<DenseSlots> {
        Arc::make_mut(&mut self.0).insert(id, slots.into())
    }

    pub(super) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Deref for ArgumentObjects {
    type Target = FxHashMap<u64, DenseSlots>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ArgumentObjects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}
impl<S: Into<DenseSlots>> FromIterator<(u64, S)> for ArgumentObjects {
    fn from_iter<T: IntoIterator<Item = (u64, S)>>(entries: T) -> Self {
        Self(Arc::new(
            entries
                .into_iter()
                .map(|(id, slots)| (id, slots.into()))
                .collect(),
        ))
    }
}
impl<S: Into<DenseSlots>> From<FxHashMap<u64, S>> for ArgumentObjects {
    fn from(entries: FxHashMap<u64, S>) -> Self {
        entries.into_iter().collect()
    }
}
impl IntoIterator for ArgumentObjects {
    type Item = (u64, DenseSlots);
    type IntoIter = std::collections::hash_map::IntoIter<u64, DenseSlots>;
    fn into_iter(self) -> Self::IntoIter {
        match Arc::try_unwrap(self.0) {
            Ok(entries) => entries.into_iter(),
            Err(entries) => entries.as_ref().clone().into_iter(),
        }
    }
}
impl<'a> IntoIterator for &'a ArgumentObjects {
    type Item = (&'a u64, &'a DenseSlots);
    type IntoIter = std::collections::hash_map::Iter<'a, u64, DenseSlots>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a> IntoIterator for &'a mut ArgumentObjects {
    type Item = (&'a u64, &'a mut DenseSlots);
    type IntoIter = std::collections::hash_map::IterMut<'a, u64, DenseSlots>;
    fn into_iter(self) -> Self::IntoIter {
        Arc::make_mut(&mut self.0).iter_mut()
    }
}

#[cfg(test)]
mod tests;
