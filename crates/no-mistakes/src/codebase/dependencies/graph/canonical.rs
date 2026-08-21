/// Reverse map from `canonicalize(path)` to the discovery spelling.
///
/// Built lazily on the first `visible_path` miss. Discovery identities already
/// live in `visible`; the reverse map exists only for symlink / case-fold
/// lookups. Eagerly filling it is a `realpath` per file and dominates
/// `GraphFiles::from_files` on large monorepos.
struct CanonicalVisible {
    cache: std::sync::Mutex<Option<HashMap<PathBuf, PathBuf>>>,
    universe: std::sync::Arc<()>,
}

impl CanonicalVisible {
    fn empty() -> Self {
        Self {
            cache: std::sync::Mutex::new(None),
            universe: std::sync::Arc::new(()),
        }
    }

    fn universe(&self) -> &std::sync::Arc<()> {
        &self.universe
    }

    fn bump_universe(&mut self) {
        self.universe = std::sync::Arc::new(());
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<HashMap<PathBuf, PathBuf>>> {
        self.cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn insert_if_built(&self, canonical: PathBuf, original: PathBuf) {
        if let Some(map) = self.lock().as_mut() {
            match map.entry(canonical) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(original);
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    // Same first-sorted-alias rule as `build_canonical_visible`.
                    if original < *entry.get() {
                        entry.insert(original);
                    }
                }
            }
        }
    }

    fn get(
        &self,
        all: &[PathBuf],
        visible: &crate::fx::PathSet,
        canonical: &Path,
    ) -> Option<PathBuf> {
        let mut guard = self.lock();
        if guard.is_none() {
            *guard = Some(build_canonical_visible(all, visible));
        }
        guard.as_ref()?.get(canonical).cloned()
    }
}

fn build_canonical_visible(
    all: &[PathBuf],
    visible: &crate::fx::PathSet,
) -> HashMap<PathBuf, PathBuf> {
    // `all` is sorted; first discovery spelling wins on a canonical collision.
    let mut map = HashMap::new();
    for path in all.iter().filter(|path| visible.contains(*path)) {
        if let Ok(canonical) = path.canonicalize() {
            map.entry(crate::codebase::ts_resolver::normalize_path(&canonical))
                .or_insert_with(|| path.clone());
        }
    }
    map
}
