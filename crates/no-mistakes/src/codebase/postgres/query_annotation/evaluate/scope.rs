use super::Value;
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

#[derive(Default)]
struct Bindings {
    values: FxHashMap<String, Value>,
    builders: OnceLock<FxHashSet<u64>>,
}
impl Clone for Bindings {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            builders: OnceLock::new(),
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

impl Scope {
    pub(super) fn contains_builders(&self, ids: &FxHashSet<u64>) -> bool {
        let builders = self.0.builders.get_or_init(|| {
            fn collect(value: &Value, ids: &mut FxHashSet<u64>) {
                match value {
                    Value::Prefix(_, _, Some(id)) => {
                        ids.insert(*id);
                    }
                    Value::Promise(value) | Value::Evaluated(value, _) => collect(value, ids),
                    Value::Aggregate(values) | Value::Possible(values) | Value::Joined(values) => {
                        for value in values {
                            collect(value, ids);
                        }
                    }
                    _ => {}
                }
            }
            let mut builders = FxHashSet::default();
            for value in self.values() {
                collect(value, &mut builders);
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
