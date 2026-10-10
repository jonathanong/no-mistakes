use super::Environment;
use crate::codebase::postgres::query_annotation::Function;
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
};

/// Immutable snapshots share nested values until a container is written.
#[derive(Clone, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct Values(Arc<Vec<Value>>);

impl From<Vec<Value>> for Values {
    fn from(values: Vec<Value>) -> Self {
        Self(Arc::new(values))
    }
}
impl Deref for Values {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Values {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}
impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|shared| (*shared).clone())
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
