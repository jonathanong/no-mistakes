/// Reverse map from `canonicalize(path)` to the discovery spelling.
///
/// Filled incrementally on `visible_path` misses. Discovery identities already
/// live in `visible`; the reverse map exists only for symlink / case-fold
/// lookups. Eagerly filling it is a `realpath` per file and dominates
/// `GraphFiles::from_files` on large monorepos.
struct CanonicalVisible {
    cache: OnceLock<dashmap::DashMap<PathBuf, PathBuf>>,
    scanned: std::sync::Mutex<usize>,
    universe: std::sync::Arc<()>,
}

impl CanonicalVisible {
    fn empty() -> Self {
        Self {
            cache: OnceLock::new(),
            scanned: std::sync::Mutex::new(0),
            universe: std::sync::Arc::new(()),
        }
    }

    fn universe(&self) -> &std::sync::Arc<()> {
        &self.universe
    }

    fn bump_universe(&mut self) {
        self.universe = std::sync::Arc::new(());
        *self.scanned.lock().expect("canonical scan mutex") = 0;
        self.cache.take();
    }

    fn insert_if_built(&self, canonical: PathBuf, original: PathBuf) {
        let Some(map) = self.cache.get() else {
            return;
        };
        insert_first_sorted_alias(map, canonical, original);
    }

    fn get(&self, all: &[PathBuf], visible: &[u8], canonical: &Path) -> Option<PathBuf> {
        let map = self.cache.get_or_init(dashmap::DashMap::new);
        if let Some(hit) = map.get(canonical) {
            return Some(hit.clone());
        }
        let mut scanned = self.scanned.lock().expect("canonical scan mutex");
        while *scanned < all.len() {
            let index = *scanned;
            *scanned += 1;
            if visible.get(index).copied() != Some(1) {
                continue;
            }
            let path = &all[index];
            let Ok(real) = path.canonicalize() else {
                continue;
            };
            let key = crate::codebase::ts_resolver::normalize_path(&real);
            let matched = key.as_path() == canonical;
            insert_first_sorted_alias(map, key, path.clone());
            if matched {
                return map.get(canonical).as_deref().cloned();
            }
        }
        map.get(canonical).as_deref().cloned()
    }
}

fn insert_first_sorted_alias(
    map: &dashmap::DashMap<PathBuf, PathBuf>,
    canonical: PathBuf,
    original: PathBuf,
) {
    match map.entry(canonical) {
        dashmap::mapref::entry::Entry::Vacant(entry) => {
            entry.insert(original);
        }
        dashmap::mapref::entry::Entry::Occupied(mut entry) => {
            if original < *entry.get() {
                entry.insert(original);
            }
        }
    }
}
