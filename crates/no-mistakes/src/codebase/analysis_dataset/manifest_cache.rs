use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

type ManifestResult<T> = Result<Arc<T>, Arc<str>>;
type ManifestCell<T> = Arc<OnceLock<ManifestResult<T>>>;
type ManifestEntries<T> = HashMap<Option<PathBuf>, ManifestCell<T>>;

pub(super) struct ManifestCache<T> {
    entries: Mutex<ManifestEntries<T>>,
}

pub(super) struct ManifestLoad<T> {
    pub(super) value: ManifestResult<T>,
    pub(super) loaded: bool,
}

impl<T> Default for ManifestCache<T> {
    fn default() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }
}

impl<T> ManifestCache<T> {
    pub(super) fn load(
        &self,
        key: Option<PathBuf>,
        load: impl FnOnce() -> ManifestResult<T>,
    ) -> ManifestLoad<T> {
        let cell = {
            let mut entries = self
                .entries
                .lock()
                .expect("analysis manifest-cache mutex must not be poisoned");
            Arc::clone(
                entries
                    .entry(key)
                    .or_insert_with(|| Arc::new(OnceLock::new())),
            )
        };
        let loaded = Cell::new(false);
        let value = cell
            .get_or_init(|| {
                loaded.set(true);
                load()
            })
            .clone();
        ManifestLoad {
            value,
            loaded: loaded.get(),
        }
    }
}
