use super::Environment;
use crate::fx::FxHashMap;
use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

pub(in crate::codebase::postgres::query_annotation) type CapturedBindings =
    FxHashMap<Environment, CapturedOrigins>;

/// Prefix owners and origins cannot move during compaction rooted at `base`.
/// Out-of-range IDs deliberately fail closed to the snapshot fallback.
pub(super) fn prefix_is_unchanged(bindings: &CapturedBindings, base: Environment) -> bool {
    bindings.iter().all(|(owner, origins)| {
        *owner < base && origins.max_origin().is_none_or(|origin| origin < base)
    })
}

mod refresh;

#[derive(Debug, Default)]
struct OriginMap {
    bindings: FxHashMap<String, Environment>,
    // Clones share this cache with the immutable origin map; branch-free runs
    // pay no scan unless an alternative actually needs the prefix proof.
    max_origin: OnceLock<Option<Environment>>,
    refresh: std::sync::Mutex<Option<refresh::State>>,
}

impl Clone for OriginMap {
    fn clone(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
            max_origin: self.max_origin.clone(),
            refresh: Default::default(),
        }
    }
}

impl PartialEq for OriginMap {
    fn eq(&self, other: &Self) -> bool {
        self.bindings == other.bindings
    }
}

impl Eq for OriginMap {}

/// Snapshots own separate reader tables but share each immutable origin map.
/// Only compaction that moves an origin needs to copy its capture names.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct CapturedOrigins(Arc<OriginMap>);

impl CapturedOrigins {
    pub(super) fn remap(&mut self, indices: &FxHashMap<Environment, Environment>) {
        if self
            .values()
            .any(|origin| indices.get(origin).is_some_and(|new| new != origin))
        {
            let origins = Arc::make_mut(&mut self.0);
            for origin in origins.bindings.values_mut() {
                if let Some(index) = indices.get(origin) {
                    *origin = *index;
                }
            }
            origins.max_origin.take();
            origins.refresh = Default::default();
        }
    }

    pub(super) fn max_origin(&self) -> Option<Environment> {
        *self
            .0
            .max_origin
            .get_or_init(|| self.0.bindings.values().copied().max())
    }
}

impl From<FxHashMap<String, Environment>> for CapturedOrigins {
    fn from(origins: FxHashMap<String, Environment>) -> Self {
        Self(Arc::new(OriginMap {
            bindings: origins,
            max_origin: OnceLock::new(),
            refresh: Default::default(),
        }))
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
        &self.0.bindings
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
