use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) fn source_store_for_files(files: &[PathBuf]) -> Arc<SourceStore> {
    Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(files))))
}

pub(crate) fn complete_source_store_for_files(files: &[PathBuf]) -> Arc<SourceStore> {
    Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(files))).with_complete_inventory())
}

pub(crate) use no_mistakes_rule_kernel::source_access::{
    checked_source, invalid_utf8, read_failure, read_language_source, unanalyzable_source,
};

pub(crate) fn suppress_invalid_utf8(
    findings: &mut Vec<super::RuleFinding>,
    error: &std::io::Error,
) {
    if let Some(error) = invalid_utf8(error) {
        super::suppress_rule_findings_with_source(
            findings,
            &String::from_utf8_lossy(error.as_bytes()),
        );
    }
}
