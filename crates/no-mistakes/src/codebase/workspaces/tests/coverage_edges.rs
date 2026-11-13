use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::codebase::ts_resolver::VisiblePathLookup;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::codebase::workspaces::{
    load_from_files, load_from_files_with_session, load_from_source_store,
    load_root_package_from_source_store, load_workspace_globs_from_files,
    load_workspace_globs_from_source_store, IndexedWorkspaceMap, WorkspaceMap, WorkspacePackage,
};

fn fixture(name: &str) -> PathBuf {
    super::saved_workspace_fixture(name)
}

fn package(dir: &Path, imports: Option<serde_json::Value>) -> WorkspacePackage {
    WorkspacePackage {
        name: "app".to_string(),
        dir: dir.to_path_buf(),
        entry: None,
        exports: None,
        imports,
    }
}

#[test]
fn workspace_manifest_edges_fail_closed() {
    let missing = fixture("no-package-json");
    let absent = [
        missing.join("package.json"),
        missing.join("pnpm-workspace.yaml"),
    ];
    for path in &absent {
        assert!(load_workspace_globs_from_files(&missing, std::slice::from_ref(path)).is_err());
    }
    let store = SourceStore::new(Arc::new(FileInventory::from_paths(&[absent[1].clone()])));
    assert!(load_workspace_globs_from_source_store(&missing, &store).is_err());

    let bad_field = fixture("bad-workspaces-field");
    let error =
        load_workspace_globs_from_files(&bad_field, &[bad_field.join("package.json")]).unwrap_err();
    assert!(error.to_string().contains("WorkspacesField"), "{error}");

    let invalid = fixture("pnpm-invalid-packages");
    let error = load_workspace_globs_from_files(&invalid, &[invalid.join("pnpm-workspace.yaml")])
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("packages field must be a list of strings"),
        "{error}"
    );

    let conflict = fixture("pnpm-package-conflict");
    let globs = load_workspace_globs_from_files(&conflict, &[conflict.join("pnpm-workspace.yaml")])
        .unwrap();
    assert!(globs.is_empty());
}

#[test]
fn blank_member_and_absent_manifests_are_omitted() {
    let root = fixture("unnamed-member");
    let files = vec![
        root.join("package.json"),
        root.join("packages/blank/package.json"),
        root.join("packages/named/package.json"),
    ];
    let loaded = load_from_files(&root, &files).unwrap();
    let without_session = load_from_files_with_session(&root, &files, None).unwrap();
    assert_eq!(loaded.packages.len(), 1);
    assert_eq!(loaded.packages[0].name, "named");
    assert_eq!(without_session.packages.len(), 1);

    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    assert_eq!(
        load_from_source_store(&root, &sources)
            .unwrap()
            .packages
            .len(),
        1
    );

    let missing = fixture("no-package-json");
    assert!(super::super::load_package(&missing, None)
        .unwrap()
        .is_none());
    let empty = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    assert!(load_root_package_from_source_store(&root, &[], &empty)
        .unwrap()
        .is_none());
}

#[test]
fn hash_imports_miss_when_the_map_or_importer_is_absent() {
    let dir = PathBuf::from("/repo/app");
    let importer = dir.join("src/a.ts");
    let missing_map = WorkspaceMap::from_packages(vec![package(&dir, None)]);
    assert_eq!(
        missing_map.resolve_specifier_from("#local", &importer),
        None
    );

    let missing_key = WorkspaceMap::from_packages(vec![package(
        &dir,
        Some(serde_json::json!({"#other": "./other.ts"})),
    )]);
    assert_eq!(
        missing_key.resolve_specifier_from("#local", &importer),
        None
    );

    let indexed = IndexedWorkspaceMap::from_packages(Vec::new());
    let visible = std::collections::HashSet::<PathBuf>::new();
    let visible: &dyn VisiblePathLookup = &visible;
    assert_eq!(
        indexed.resolve_specifier_from_visible("#local", visible),
        None
    );
}
