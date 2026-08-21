use super::SourceStore;
use std::path::Path;
use std::sync::Arc;

impl SourceStore {
    /// Read through a prepared store, falling back to a one-shot filesystem
    /// read only when tests invoke a helper without a session.
    pub(crate) fn read_optional(sources: Option<&Self>, path: &Path) -> Option<Arc<str>> {
        match sources {
            Some(store) => store.read_path(path).ok(),
            None => std::fs::read_to_string(path).ok().map(Arc::from),
        }
    }

    /// Parse JSON through a prepared store, with a test-only filesystem fallback.
    pub(crate) fn parse_json_optional(
        sources: Option<&Self>,
        path: &Path,
    ) -> Option<Arc<serde_json::Value>> {
        match sources {
            Some(store) => store.parse_json_path(path).ok(),
            None => std::fs::read_to_string(path)
                .ok()
                .and_then(|source| serde_json::from_str(&source).ok())
                .map(Arc::new),
        }
    }
}
