use super::Environment;
use crate::fx::FxHashMap;
use std::{ops::Deref, sync::Arc};

pub(in crate::codebase::postgres::query_annotation) type CapturedBindings =
    FxHashMap<Environment, CapturedOrigins>;

/// Snapshots own separate reader tables but share each immutable origin map.
/// Only compaction that moves an origin needs to copy its capture names.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct CapturedOrigins(
    Arc<FxHashMap<String, Environment>>,
);

impl CapturedOrigins {
    pub(super) fn remap(&mut self, indices: &FxHashMap<Environment, Environment>) {
        if self
            .values()
            .any(|origin| indices.get(origin).is_some_and(|new| new != origin))
        {
            for origin in Arc::make_mut(&mut self.0).values_mut() {
                if let Some(index) = indices.get(origin) {
                    *origin = *index;
                }
            }
        }
    }
}

impl From<FxHashMap<String, Environment>> for CapturedOrigins {
    fn from(origins: FxHashMap<String, Environment>) -> Self {
        Self(Arc::new(origins))
    }
}

impl FromIterator<(String, Environment)> for CapturedOrigins {
    fn from_iter<T: IntoIterator<Item = (String, Environment)>>(origins: T) -> Self {
        FxHashMap::from_iter(origins).into()
    }
}

impl Deref for CapturedOrigins {
    type Target = FxHashMap<String, Environment>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a CapturedOrigins {
    type Item = (&'a String, &'a Environment);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Environment>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests;
