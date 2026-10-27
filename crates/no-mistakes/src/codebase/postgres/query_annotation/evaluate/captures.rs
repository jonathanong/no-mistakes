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
        *owner < base && origins.max_origin().map_or(true, |origin| origin < base)
    })
}

#[derive(Clone, Debug, Default)]
struct OriginMap {
    bindings: FxHashMap<String, Environment>,
    // Clones share this cache with the immutable origin map; branch-free runs
    // pay no scan unless an alternative actually needs the prefix proof.
    max_origin: OnceLock<Option<Environment>>,
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
        }
    }

    pub(super) fn max_origin(&self) -> Option<Environment> {
        *self
            .0
            .max_origin
            .get_or_init(|| self.0.bindings.values().copied().max())
    }

    #[cfg(test)]
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn strong_count_probe(
        &self,
    ) -> impl Fn() -> usize {
        let weak = Arc::downgrade(&self.0);
        move || weak.strong_count()
    }

    #[cfg(test)]
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn insert_origin(
        &mut self,
        name: String,
        origin: Environment,
    ) {
        let origins = Arc::make_mut(&mut self.0);
        origins.bindings.insert(name, origin);
        origins.max_origin.take();
    }

    #[cfg(test)]
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn reserve_origins(
        &mut self,
        capacity: usize,
    ) {
        Arc::make_mut(&mut self.0).bindings.reserve(capacity);
    }
}

impl From<FxHashMap<String, Environment>> for CapturedOrigins {
    fn from(origins: FxHashMap<String, Environment>) -> Self {
        Self(Arc::new(OriginMap {
            bindings: origins,
            max_origin: OnceLock::new(),
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
