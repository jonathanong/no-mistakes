use super::{references::Edges, Value};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

#[derive(Default)]
struct Data {
    values: Vec<Value>,
    references: OnceLock<Edges>,
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Self {
            values: self.copy_values(),
            references: OnceLock::new(),
        }
    }
}

impl Data {
    fn copy_values(&self) -> Vec<Value> {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(
                "postgres.annotation.dense_arena_slots_copied",
                self.values.len() as u64,
            );
        }
        self.values.clone()
    }
}

/// Arm snapshots share dense arguments; writes detach and invalidate direct edges.
#[derive(Clone, Default)]
pub(in crate::codebase::postgres::query_annotation) struct DenseSlots(Arc<Data>);

impl DenseSlots {
    pub(super) fn references(&self) -> &Edges {
        self.0.references.get_or_init(|| {
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment(
                    "postgres.annotation.dense_reference_slots_scanned",
                    self.len() as u64,
                );
            }
            Edges::collect(self.iter())
        })
    }

    pub(super) fn shared(&self) -> bool {
        Arc::strong_count(&self.0) > 1
    }
}

impl PartialEq for DenseSlots {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.values == other.0.values
    }
}
impl Eq for DenseSlots {}
impl From<DenseSlots> for super::value::Values {
    fn from(slots: DenseSlots) -> Self {
        slots.into_iter().collect::<Vec<_>>().into()
    }
}
impl From<Vec<Value>> for DenseSlots {
    fn from(values: Vec<Value>) -> Self {
        Self(Arc::new(Data {
            values,
            references: OnceLock::new(),
        }))
    }
}
impl Deref for DenseSlots {
    type Target = Vec<Value>;
    fn deref(&self) -> &Self::Target {
        &self.0.values
    }
}
impl DerefMut for DenseSlots {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let data = Arc::make_mut(&mut self.0);
        data.references.take();
        &mut data.values
    }
}
impl IntoIterator for DenseSlots {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;
    fn into_iter(self) -> Self::IntoIter {
        Arc::try_unwrap(self.0)
            .map(|data| data.values)
            .unwrap_or_else(|shared| shared.copy_values())
            .into_iter()
    }
}
impl<'a> IntoIterator for &'a DenseSlots {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut DenseSlots {
    type Item = &'a mut Value;
    type IntoIter = std::slice::IterMut<'a, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests;
