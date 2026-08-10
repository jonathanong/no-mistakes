use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn source_store_for_files(files: &[PathBuf]) -> Arc<SourceStore> {
    Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(files))))
}

pub(crate) fn read_source(sources: &SourceStore, path: &Path) -> Option<Arc<str>> {
    sources.read_path(path).ok()
}
