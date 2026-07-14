use super::SourceStore;
use crate::codebase::ts_source::FileInventory;
use rayon::prelude::*;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/analysis-dataset/source-store")
        .join(path)
}

#[test]
fn successful_reads_are_exact_and_memoized_across_threads() {
    let path = fixture("alpha.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let store = SourceStore::new(inventory);

    let sources = (0..16)
        .into_par_iter()
        .map(|_| store.read_path(&path).unwrap().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(&*sources[0], "export const alpha = \"α\";\n");
    assert!(sources
        .iter()
        .all(|source| Arc::ptr_eq(source, &sources[0])));
    assert_eq!(store.physical_read_count(), 1);
}

#[test]
fn failed_reads_are_memoized_with_the_original_io_kind() {
    let missing = fixture("missing.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&missing)));
    let id = inventory.id_for_path(&missing).unwrap();
    let store = SourceStore::new(inventory);

    let first = store.read(id).unwrap().unwrap_err();
    let second = store.read(id).unwrap().unwrap_err();

    assert_eq!(first.kind(), ErrorKind::NotFound);
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(store.physical_read_count(), 1);
}

#[test]
fn strict_utf8_read_errors_are_cached() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/shared-facts/fixture/src/unreadable.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&directory)));
    let store = SourceStore::new(inventory);

    let first = store.read_path(&directory).unwrap().unwrap_err();
    let second = store.read_path(&directory).unwrap().unwrap_err();

    assert!(matches!(
        first.kind(),
        ErrorKind::IsADirectory | ErrorKind::PermissionDenied
    ));
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(store.physical_read_count(), 1);
}

#[test]
fn supplemental_paths_are_memoized_without_changing_the_frozen_inventory() {
    let known = fixture("alpha.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&known)));
    let store = SourceStore::new(Arc::clone(&inventory));
    let supplemental = fixture("beta.ts");

    assert!(Arc::ptr_eq(store.inventory(), &inventory));
    let first = store.read_path(&supplemental).unwrap().unwrap();
    let second = store.read_path(&supplemental).unwrap().unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(store.inventory().len(), 1);
    assert_eq!(store.physical_read_count(), 1);
}

#[test]
fn json_load_failures_retain_io_and_syntax_causes() {
    let missing = fixture("missing.json");
    let malformed = fixture("alpha.ts");
    let unreadable = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/shared-facts/fixture/src/unreadable.ts");
    let inventory = Arc::new(FileInventory::from_paths(&[
        missing.clone(),
        malformed.clone(),
        unreadable.clone(),
    ]));
    let store = SourceStore::new(inventory);

    let first_missing = store.parse_json_path(&missing).unwrap().unwrap_err();
    let second_missing = store.parse_json_path(&missing).unwrap().unwrap_err();
    let super::JsonLoadError::Io(first_io) = first_missing else {
        panic!("missing JSON must retain its filesystem error");
    };
    let super::JsonLoadError::Io(second_io) = second_missing else {
        panic!("cached missing JSON must retain its filesystem error");
    };
    assert_eq!(first_io.kind(), ErrorKind::NotFound);
    assert!(Arc::ptr_eq(&first_io, &second_io));

    let super::JsonLoadError::Syntax(error) =
        store.parse_json_path(&malformed).unwrap().unwrap_err()
    else {
        panic!("malformed JSON must retain its syntax error");
    };
    assert_ne!(
        error.to_string(),
        "EOF while parsing a value at line 1 column 0"
    );

    let super::JsonLoadError::Io(error) = store.parse_json_path(&unreadable).unwrap().unwrap_err()
    else {
        panic!("unreadable JSON must retain its filesystem error");
    };
    assert!(matches!(
        error.kind(),
        ErrorKind::IsADirectory | ErrorKind::PermissionDenied
    ));
    assert_eq!(store.physical_read_count(), 3);
    assert_eq!(store.json_parse_count(), 3);
}

#[test]
fn workspace_root_error_reports_cached_missing_manifest_io_cause() {
    let manifest = fixture("missing-root/package.json");
    let root = manifest.parent().unwrap();
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&manifest)));
    let store = SourceStore::new(inventory);

    let error = crate::codebase::workspaces::load_from_source_store(root, &store)
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("failed to load workspace manifest"),
        "{error}"
    );
    assert!(error.contains("package.json"), "{error}");
    assert!(!error.contains("EOF while parsing"), "{error}");
    assert_eq!(store.physical_read_count(), 1);
}

#[test]
fn package_manifest_json_is_parsed_once_across_workspace_and_rule_consumers() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/workspace-symbol/package.json");
    let root = manifest.parent().unwrap();
    let files = vec![manifest.clone()];
    let inventory = Arc::new(FileInventory::from_paths(&files));
    let store = SourceStore::new(inventory);

    let package =
        crate::codebase::workspaces::load_root_package_from_source_store(root, &files, &store)
            .unwrap();
    assert!(package.is_some());
    let _dependencies = crate::codebase::package_deps::dependency_entries_from_source_store(
        &manifest,
        crate::codebase::package_deps::ALL_DEPENDENCY_FIELDS,
        &store,
    );

    assert_eq!(store.json_parse_count(), 1);
    assert_eq!(store.physical_read_count(), 1);
}
