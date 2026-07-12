use anyhow::Result;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

type CachedRouteImports = std::result::Result<Arc<Vec<PathBuf>>, Arc<String>>;
pub(super) type RouteImportCache = DashMap<PathBuf, Arc<OnceLock<CachedRouteImports>>>;

pub(super) fn get_or_compute_route_imports(
    import_cache: &RouteImportCache,
    normalized_path: PathBuf,
    compute: impl FnOnce(&Path) -> Result<Vec<PathBuf>>,
) -> Result<Arc<Vec<PathBuf>>> {
    // Install only a cheap synchronization cell while holding DashMap's shard
    // lock. The expensive read and parse then run outside that lock, once per
    // path even when many route traversals reach the same module concurrently.
    let cache_entry = import_cache
        .entry(normalized_path.clone())
        .or_default()
        .clone();
    cache_entry
        .get_or_init(|| {
            compute(&normalized_path)
                .map(Arc::new)
                .map_err(|error| Arc::new(format!("{error:#}")))
        })
        .clone()
        .map_err(|error| anyhow::Error::msg(error.as_str().to_string()))
}
