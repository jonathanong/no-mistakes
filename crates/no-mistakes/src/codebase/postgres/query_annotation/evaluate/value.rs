use super::Environment;
use crate::codebase::postgres::query_annotation::Function;
use crate::fx::FxHashSet;
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, OnceLock},
};

/// Immutable snapshots share nested values until a container is written.
struct Data {
    values: Vec<Value>,
    contains_reference: OnceLock<bool>,
    environments: OnceLock<FxHashSet<Environment>>,
}
impl Clone for Data {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            contains_reference: OnceLock::new(),
            environments: OnceLock::new(),
        }
    }
}

/// Equality depends on semantic values, never on whether the memo has run.
#[derive(Clone)]
pub(in crate::codebase::postgres::query_annotation) struct Values(Arc<Data>);

impl PartialEq for Values {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.values == other.0.values
    }
}
impl Eq for Values {}

impl Values {
    pub(super) fn contains_environment(&self) -> bool {
        !self.environment_indices().is_empty()
    }

    pub(super) fn environment_indices(&self) -> &FxHashSet<Environment> {
        self.0.environments.get_or_init(|| {
            fn collect(value: &Value, environments: &mut FxHashSet<Environment>) {
                match value {
                    Value::Function(_, _, env) => {
                        environments.insert(*env);
                    }
                    Value::Promise(value) | Value::Evaluated(value, _) => {
                        collect(value, environments)
                    }
                    Value::Aggregate(values) | Value::Joined(values) | Value::Possible(values) => {
                        environments.extend(values.environment_indices());
                    }
                    _ => {}
                }
            }
            let mut environments = FxHashSet::default();
            for value in &self.0.values {
                collect(value, &mut environments);
            }
            environments
        })
    }

    pub(super) fn identity(&self) -> *const () {
        Arc::as_ptr(&self.0).cast()
    }

    pub(super) fn contains_reference(&self) -> bool {
        *self.0.contains_reference.get_or_init(|| {
            fn references(value: &Value) -> bool {
                match value {
                    Value::Prefix(_, _, Some(_)) | Value::Arguments(_) => true,
                    Value::Promise(value) | Value::Evaluated(value, _) => references(value),
                    Value::Aggregate(values) | Value::Joined(values) | Value::Possible(values) => {
                        values.contains_reference()
                    }
                    _ => false,
                }
            }
            self.0.values.iter().any(references)
        })
    }
}

impl From<Vec<Value>> for Values {
    fn from(values: Vec<Value>) -> Self {
        Self(Arc::new(Data {
            values,
            contains_reference: OnceLock::new(),
            environments: OnceLock::new(),
        }))
    }
}
impl Deref for Values {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0.values
    }
}
impl DerefMut for Values {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let data = Arc::make_mut(&mut self.0);
        data.contains_reference.take();
        data.environments.take();
        &mut data.values
    }
}
impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        Arc::try_unwrap(self.0)
            .map(|data| data.values)
            .unwrap_or_else(|shared| shared.values.clone())
            .into_iter()
    }
}
impl<'a> IntoIterator for &'a Values {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut Values {
    type Item = &'a mut Value;
    type IntoIter = std::slice::IterMut<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) enum Value {
    Prefix(String, bool, Option<u64>),
    Promise(Box<Value>),
    Evaluated(Box<Value>, bool),
    Aggregate(Values),
    // Alternative binding values; unlike Aggregate, this is not a container.
    Joined(Values),
    // Candidate runtime values with an implicit unknown alternative.
    Possible(Values),
    Arguments(u64),
    Function(Arc<Function>, PathBuf, Environment),
    Unknown,
    Primitive,
    Unsupported,
    SlotDeletion,
}
impl Value {
    pub(super) fn exposed(self) -> Self {
        let mut value = self;
        while let Self::Evaluated(inner, _) = value {
            value = *inner;
        }
        value
    }
}

#[cfg(test)]
mod tests;
