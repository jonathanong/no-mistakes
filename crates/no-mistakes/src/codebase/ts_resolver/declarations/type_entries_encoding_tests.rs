use super::package_entry;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn declaration_entries_accept_lossy_source_without_another_physical_read() {
    let root = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/symbols/signature-impact-encoding/declarations"),
    );
    let path = root.join("index.d.ts");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    assert!(sources.read_path(&path).is_err());
    assert!(matches!(
        package_entry(&root, &sources),
        super::PackageEntry::File(found) if found == path
    ));
    assert_eq!(
        super::resolve_name("./index.d.ts", &root, None, &path, &root, &sources),
        Some(path.clone())
    );
    assert_eq!(
        super::package::resolve_in_context("./index.d.ts", &path, &root, &sources),
        Some(path.clone())
    );
    assert!(sources
        .read_source_path(&path)
        .unwrap()
        .contains("WorkerError"));
    assert!(sources.read_path(&path).is_err());
    assert_eq!(sources.physical_read_count(), 2);
}
