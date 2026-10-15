use super::*;

impl ParameterCache {
    pub(in super::super) fn type_origin(
        &self,
        key: (std::path::PathBuf, String),
        resolve: impl FnOnce() -> (String, String),
    ) -> (String, String) {
        // Keep cold shard construction small; resolutions run outside guards,
        // so the shard count does not cap concurrent independent resolutions.
        let origins = self
            .type_origins
            .get_or_init(|| DashMap::with_hasher_and_shard_amount(FxBuildHasher, 8));
        if let Some(value) = origins.get(&key).and_then(|slot| slot.get().cloned()) {
            return value;
        }
        let slot = Arc::clone(origins.entry(key).or_default().value());
        // Release the shard guard before walking prepared type re-exports. Even
        // unrelated keys in the same shard must be able to resolve concurrently.
        slot.get_or_init(resolve).clone()
    }
}
