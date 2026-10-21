use super::dense_slots::DenseSlots;
use crate::fx::FxHashMap;
use std::ops::{Deref, DerefMut};

/// Mutable identities retain shared immutable dense snapshots between writes.
#[derive(Clone, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct ArgumentObjects(
    FxHashMap<u64, DenseSlots>,
);

impl ArgumentObjects {
    pub(super) fn insert(&mut self, id: u64, slots: impl Into<DenseSlots>) -> Option<DenseSlots> {
        self.0.insert(id, slots.into())
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
        &mut self.0
    }
}
impl<S: Into<DenseSlots>> FromIterator<(u64, S)> for ArgumentObjects {
    fn from_iter<T: IntoIterator<Item = (u64, S)>>(entries: T) -> Self {
        Self(
            entries
                .into_iter()
                .map(|(id, slots)| (id, slots.into()))
                .collect(),
        )
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
        self.0.into_iter()
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
        self.0.iter_mut()
    }
}

#[cfg(test)]
mod tests;
