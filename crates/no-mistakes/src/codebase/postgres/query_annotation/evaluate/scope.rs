use super::Value;
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

#[derive(Default)]
struct Bindings {
    values: FxHashMap<String, Value>,
    builders: OnceLock<BuilderIndex>,
}
// An empty builder set alone cannot prove the absence of lexical/heap roots.
enum BuilderIndex {
    ModuleOnly,
    Builders(FxHashSet<u64>),
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
    /// Only completed inert templates may install this proof. Every mutable
    /// Scope view clears the same cache before exposing its binding values.
    pub(super) fn mark_module_only(&mut self) {
        if self.values().all(|value| {
            matches!(
                value,
                Value::Unknown
                    | Value::Unsupported
                    | Value::Number(_)
                    | Value::Boolean(_)
                    | Value::Prefix(_, _, None)
                    | Value::Function(_, _, super::Capture::Module)
            )
        }) {
            Arc::make_mut(&mut self.0).builders = OnceLock::from(BuilderIndex::ModuleOnly);
        }
    }

    pub(super) fn is_module_only(&self) -> bool {
        matches!(self.0.builders.get(), Some(BuilderIndex::ModuleOnly))
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
            BuilderIndex::Builders(builders)
        });
        match builders {
            BuilderIndex::ModuleOnly => false,
            BuilderIndex::Builders(builders) => !builders.is_disjoint(ids),
        }
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

#[cfg(test)]
mod frontier_tests;

#[cfg(test)]
pub(in crate::codebase::postgres::query_annotation::evaluate) mod frontier_fixture;
