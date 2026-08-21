use super::SourceStore;
use crate::codebase::ts_source::FileInventory;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl SourceStore {
    /// Read through a prepared store, or a one-file store when tests omit a session.
    pub(crate) fn read_optional(sources: Option<&Self>, path: &Path) -> Option<Arc<str>> {
        match sources {
            Some(store) => store.read_path(path).ok(),
            None => one_file_store(path).read_path(path).ok(),
        }
    }

    /// Parse JSON through a prepared store, or a one-file store when tests omit a session.
    pub(crate) fn parse_json_optional(
        sources: Option<&Self>,
        path: &Path,
    ) -> Option<Arc<serde_json::Value>> {
        match sources {
            Some(store) => store.parse_json_path(path).ok(),
            None => one_file_store(path).parse_json_path(path).ok(),
        }
    }
}

fn one_file_store(path: &Path) -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(&[PathBuf::from(path)])))
}
