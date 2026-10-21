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
impl PartialEq for Bindings {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Eq for Bindings {}

/// Request-local snapshots share immutable bindings and their builder index.
/// Any write detaches the bindings and invalidates the index before mutation.
#[derive(Clone, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct Scope(Arc<Bindings>);

#[cfg(not(test))]
mod binding_scan_count;
#[cfg(test)]
#[path = "scope/binding_scan_count/tests.rs"]
mod binding_scan_count;

impl Scope {
    /// Preserve map iteration order while scanning each immutable snapshot once.
    /// Writes invalidate names; values stay borrowed from the same snapshot.
    pub(super) fn reference_bindings(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0
            .reference_binding_names
            .get_or_init(|| {
                binding_scan_count::record(self.len());
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

    pub(super) fn contains_builders(&self, ids: &FxHashSet<u64>) -> bool {
        let builders = self.0.builders.get_or_init(|| {
            fn collect(
                value: &Value,
                ids: &mut FxHashSet<u64>,
                visited: &mut FxHashSet<*const ()>,
            ) {
                match value {
                    Value::Prefix(_, _, Some(id)) => {
                        ids.insert(*id);
                    }
                    Value::Promise(value) | Value::Evaluated(value, _) => {
                        collect(value, ids, visited)
                    }
                    Value::Aggregate(values)
                    | Value::References(values)
                    | Value::Object(values)
                    | Value::Record(values, _)
                    | Value::Joined(values)
                    | Value::Possible(values)
                        if values.contains_reference() && visited.insert(values.identity()) =>
                    {
                        for value in values {
                            collect(value, ids, visited);
                        }
                    }
                    _ => {}
                }
            }
            let mut builders = FxHashSet::default();
            let mut visited = FxHashSet::default();
            for value in self.values() {
                collect(value, &mut builders, &mut visited);
            }
            builders
        });
        !builders.is_disjoint(ids)
    }
}
impl From<FxHashMap<String, Value>> for Scope {
    fn from(values: FxHashMap<String, Value>) -> Self {
        Self(Arc::new(Bindings {
            values,
            builders: OnceLock::new(),
            contains_reference: OnceLock::new(),
            references: OnceLock::new(),
            reference_binding_names: OnceLock::new(),
        }))
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
        let bindings = Arc::make_mut(&mut self.0);
        bindings.builders.take();
        bindings.contains_reference.take();
        bindings.references.take();
        bindings.reference_binding_names.take();
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
