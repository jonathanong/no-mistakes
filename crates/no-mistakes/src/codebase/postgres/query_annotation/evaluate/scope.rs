use super::{references::Edges, Value};
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

#[derive(Default)]
struct Bindings {
    values: FxHashMap<String, Value>,
    builders: OnceLock<FxHashSet<u64>>,
    contains_reference: OnceLock<bool>,
    references: OnceLock<Edges>,
    reference_binding_names: OnceLock<Vec<String>>,
}
impl Clone for Bindings {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            builders: OnceLock::new(),
            contains_reference: OnceLock::new(),
            references: OnceLock::new(),
            reference_binding_names: OnceLock::new(),
        }
    }
}
impl Bindings {
    fn invalidate(&mut self) {
        self.builders.take();
        self.contains_reference.take();
        self.references.take();
        self.reference_binding_names.take();
    }
}
impl PartialEq for Bindings {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Eq for Bindings {}

/// Request-local snapshots share immutable bindings and their builder index.
/// Any write detaches the bindings and invalidates the index before mutation.
#[derive(Clone, Default)]
pub(in crate::codebase::postgres::query_annotation) struct Scope(Arc<Bindings>, changes::Changes);

#[cfg(not(test))]
#[macro_use]
mod binding_scan_count;
#[cfg(test)]
#[path = "scope/binding_scan_count/tests.rs"]
#[macro_use]
mod binding_scan_count;

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Scope {}

mod builders;
mod changes;
mod mutation;

impl Scope {
    /// Preserve map iteration order while scanning each immutable snapshot once.
    /// Writes invalidate names; values stay borrowed from the same snapshot.
    pub(super) fn reference_bindings(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0
            .reference_binding_names
            .get_or_init(|| {
                record_binding_scan!(self.len());
                self.iter()
                    .filter(|(_, value)| value.contains_reference())
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .iter()
            .map(|name| (name, &self.0.values[name]))
    }

    /// Argument/array identities remain references even when their current arena is empty.
    pub(super) fn contains_reference(&self) -> bool {
        *self
            .0
            .contains_reference
            .get_or_init(|| self.0.values.values().any(Value::contains_reference))
    }

    pub(super) fn same_bindings(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// Borrow an existing summary without initializing cold snapshots.
    pub(super) fn cached_references(&self) -> Option<&Edges> {
        self.0.references.get()
    }

    pub(super) fn references(&self) -> &Edges {
        self.0.references.get_or_init(|| {
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.prefix_scope_scans", 1);
                observer.increment(
                    "postgres.annotation.prefix_binding_scans",
                    self.len() as u64,
                );
            }
            Edges::collect(self.values())
        })
    }
}
impl From<FxHashMap<String, Value>> for Scope {
    fn from(values: FxHashMap<String, Value>) -> Self {
        Self(
            Arc::new(Bindings {
                values,
                builders: OnceLock::new(),
                contains_reference: OnceLock::new(),
                references: OnceLock::new(),
                reference_binding_names: OnceLock::new(),
            }),
            Default::default(),
        )
    }
}
impl Deref for Scope {
    type Target = FxHashMap<String, Value>;
    fn deref(&self) -> &Self::Target {
        &self.0.values
    }
}
impl DerefMut for Scope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1.clear();
        let bindings = Arc::make_mut(&mut self.0);
        bindings.invalidate();
        &mut bindings.values
    }
}
impl<'a> IntoIterator for &'a Scope {
    type Item = (&'a String, &'a Value);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut Scope {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod reference_tests;

#[cfg(test)]
mod reference_binding_tests;

#[cfg(test)]
mod precise_binding_tests;
