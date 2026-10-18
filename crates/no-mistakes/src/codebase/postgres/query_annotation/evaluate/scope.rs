use super::Value;
use crate::fx::FxHashMap;
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Default)]
struct Bindings {
    values: FxHashMap<String, Value>,
}
impl PartialEq for Bindings {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Eq for Bindings {}

/// Request-local snapshots share immutable bindings until a write detaches them.
#[derive(Clone, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct Scope(Arc<Bindings>);

impl Scope {
    pub(super) fn restore_binding(&mut self, name: String, value: Option<Value>) {
        let bindings = Arc::make_mut(&mut self.0);
        if let Some(value) = value {
            bindings.values.insert(name, value);
        } else {
            bindings.values.remove(&name);
        }
    }
}
impl From<FxHashMap<String, Value>> for Scope {
    fn from(values: FxHashMap<String, Value>) -> Self {
        Self(Arc::new(Bindings { values }))
    }
}
impl Deref for Scope {
    type Target = FxHashMap<String, Value>;
    fn deref(&self) -> &Self::Target {
        &self.0.values
    }
}
impl<'a> IntoIterator for &'a Scope {
    type Item = (&'a String, &'a Value);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[cfg(test)]
mod tests;
