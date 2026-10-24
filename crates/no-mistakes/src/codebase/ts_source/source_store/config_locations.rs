use crate::config::source_locations::ConfigLocations;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

type LocationSlot = Arc<OnceLock<Option<Arc<ConfigLocations>>>>;
pub(super) type LocationCache = OnceLock<DashMap<PathBuf, LocationSlot>>;

impl super::SourceStore {
    pub(crate) fn config_locations(&self, path: &Path) -> Option<Arc<ConfigLocations>> {
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        let slots = self.config_locations.get_or_init(DashMap::new);
        let slot = super::once_lock_slot(slots, path.clone());
        slot.get_or_init(|| {
            self.increment("config_locations.parses", 1);
            let source = self.read_path(&path).ok()?;
            ConfigLocations::parse(&source, &path).map(Arc::new)
        })
        .clone()
    }
}

#[cfg(test)]
mod tests;
