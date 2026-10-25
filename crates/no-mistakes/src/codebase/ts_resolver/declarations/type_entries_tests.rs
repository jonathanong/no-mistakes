use super::package_entry;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/type-root-entries"),
    )
}

fn entry(root: &Path, name: &str) -> Option<PathBuf> {
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    package_entry(&root.join(name), &sources)
}

#[test]
fn type_root_package_entry_follows_manifest_types_then_index_files() {
    let root = fixture();
    let expect = |dir: &str, file: &str| {
        assert_eq!(
            entry(&root, dir),
            Some(normalize_path(&root.join(dir).join(file))),
            "{dir}"
        );
    };
    expect("types-field", "worker.d.ts");
    expect("typings-field", "worker.d.ts");
    expect("index-after-manifest", "index.d.ts");
    expect("non-string-types", "index.d.ts");
    expect("bad-manifest", "index.d.ts");
    expect("index-mts", "index.d.mts");
    expect("index-cts", "index.d.cts");
    assert_eq!(entry(&root, "missing-types"), None);
    assert_eq!(entry(&root, "no-entry"), None);
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    assert_eq!(package_entry(&root.join("not-a-dir.txt"), &sources), None);
}
