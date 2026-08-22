use super::AnalysisSession;
use crate::codebase::ts_resolver::normalize_path;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;

/// Request-scoped intern table for graph node paths and names.
///
/// Owned by [`AnalysisSession`]. Duplicate normalized paths and strings share
/// one `Arc` for the lifetime of the request. Never process-global, never
/// persisted.
#[derive(Default)]
pub struct PathInterner {
    paths: DashMap<Arc<Path>, ()>,
    strings: DashMap<Arc<str>, ()>,
}

impl PathInterner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the request-owned `Arc<Path>` for `path`.
    ///
    /// Look up the borrowed path first so already-normalized inventory and
    /// discovery paths do not allocate a `PathBuf` on cache hits. Unnormalized
    /// spellings still go through [`normalize_path`] before insert.
    pub fn intern_path(&self, path: impl AsRef<Path>) -> Arc<Path> {
        let path = path.as_ref();
        if let Some(hit) = self.paths.get(path) {
            return Arc::clone(hit.key());
        }
        let normalized = normalize_path(path);
        if normalized.as_path() != path {
            if let Some(hit) = self.paths.get(normalized.as_path()) {
                return Arc::clone(hit.key());
            }
        }
        self.insert_path_arc(Arc::<Path>::from(normalized))
    }

    /// Return the request-owned `Arc<str>` for `value`.
    ///
    /// Look up the borrowed `&str` first so repeated resolver and Playwright
    /// catalog specifiers do not allocate an `Arc` on cache hits.
    pub fn intern_str(&self, value: impl AsRef<str>) -> Arc<str> {
        let value = value.as_ref();
        if let Some(hit) = self.strings.get(value) {
            return Arc::clone(hit.key());
        }
        self.insert_str_arc(Arc::from(value))
    }

    pub(crate) fn insert_path_arc(&self, interned: Arc<Path>) -> Arc<Path> {
        match self.paths.entry(Arc::clone(&interned)) {
            Entry::Occupied(entry) => Arc::clone(entry.key()),
            Entry::Vacant(entry) => {
                entry.insert(());
                interned
            }
        }
    }

    pub(crate) fn insert_str_arc(&self, interned: Arc<str>) -> Arc<str> {
        match self.strings.entry(Arc::clone(&interned)) {
            Entry::Occupied(entry) => Arc::clone(entry.key()),
            Entry::Vacant(entry) => {
                entry.insert(());
                interned
            }
        }
    }
}

impl AnalysisSession {
    /// Borrow the request-owned intern table.
    pub fn interner(&self) -> &PathInterner {
        &self.interner
    }

    /// Clone the request-owned intern table handle for graph workers.
    pub(crate) fn interner_arc(&self) -> Arc<PathInterner> {
        Arc::clone(&self.interner)
    }
}

#[cfg(test)]
mod tests;
