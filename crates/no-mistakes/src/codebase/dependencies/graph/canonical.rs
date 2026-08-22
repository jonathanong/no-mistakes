/// Reverse map from `canonicalize(path)` to the discovery spelling.
///
/// Built lazily on the first `visible_path` miss. Discovery identities already
/// live in `visible`; the reverse map exists only for symlink / case-fold
/// lookups. Eagerly filling it is a `realpath` per file and dominates
/// `GraphFiles::from_files` on large monorepos.
struct CanonicalVisible {
    cache: OnceLock<dashmap::DashMap<PathBuf, PathBuf>>,
    universe: std::sync::Arc<()>,
}

impl CanonicalVisible {
    fn empty() -> Self {
        Self {
            cache: OnceLock::new(),
            universe: std::sync::Arc::new(()),
        }
    }

    fn universe(&self) -> &std::sync::Arc<()> {
        &self.universe
    }

    fn bump_universe(&mut self) {
        self.universe = std::sync::Arc::new(());
    }

    fn insert_if_built(&self, canonical: PathBuf, original: PathBuf) {
        let Some(map) = self.cache.get() else {
            return;
        };
        match map.entry(canonical) {
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(original);
            }
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                // Same first-sorted-alias rule as `build_canonical_visible`.
                if original < *entry.get() {
                    entry.insert(original);
                }
            }
        }
    }

    fn get(&self, all: &[PathBuf], visible: &[u8], canonical: &Path) -> Option<PathBuf> {
        let map = self.cache.get_or_init(|| {
            build_canonical_visible(all, visible)
                .into_iter()
                .collect()
        });
        map.get(canonical).as_deref().cloned()
    }
}

fn build_canonical_visible(all: &[PathBuf], visible: &[u8]) -> HashMap<PathBuf, PathBuf> {
    // `all` is sorted; first discovery spelling wins on a canonical collision.
    let mut map = HashMap::new();
    for (path, flag) in all.iter().zip(visible.iter()) {
        if *flag == 0 {
            continue;
        }
        if let Ok(canonical) = path.canonicalize() {
            map.entry(crate::codebase::ts_resolver::normalize_path(&canonical))
                .or_insert_with(|| path.clone());
        }
    }
    map
}
