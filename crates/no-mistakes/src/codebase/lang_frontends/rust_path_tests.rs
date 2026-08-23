use super::rust_path::{cargo_path_deps, path_attr_mods};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn store_for(files: &[PathBuf]) -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(files)))
}

#[test]
fn cargo_path_deps_skips_missing_manifests() {
    let store = store_for(&[]);
    assert!(cargo_path_deps(&store, Path::new("/missing/Cargo.toml")).is_empty());
}

#[test]
fn path_attr_mods_resolve_relative_targets() {
    let file = Path::new("/repo/src/lib.rs");
    let paths = path_attr_mods("#[path = \"alt.rs\"]\nmod alt;", file);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("src/alt.rs"));
    assert!(path_attr_mods("mod alt;", file).is_empty());
}
