use super::SourceStore;
use crate::codebase::ts_source::FileInventory;
use rayon::prelude::*;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};

const CONCURRENT_CALLERS: usize = 16;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/analysis-dataset/source-store")
        .join(path)
}

#[test]
fn successful_reads_are_exact_and_memoized_across_threads() {
    let path = fixture("alpha.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let store = SourceStore::new_observed(inventory, Some(Arc::clone(&observer)));

    let sources = (0..16)
        .into_par_iter()
        .map(|_| store.read_path(&path).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(&*sources[0], "export const alpha = \"α\";\n");
    assert!(sources
        .iter()
        .all(|source| Arc::ptr_eq(source, &sources[0])));
    assert_eq!(store.physical_read_count(), 1);
    assert_eq!(
        observer.source_read_snapshot()[&crate::codebase::ts_resolver::normalize_path(&path)],
        1
    );
    let work = observer.snapshot().work;
    assert_eq!(work["source.requests"], 16);
    assert_eq!(work["source.reads"], 1);
    assert_eq!(work["source.cache_hits"], 15);
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

    let first = store.read_path(&directory).unwrap_err();
    let second = store.read_path(&directory).unwrap_err();

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
    let first = store.read_path(&supplemental).unwrap();
    let second = store.read_path(&supplemental).unwrap();
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

    let first_missing = store.parse_json_path(&missing).unwrap_err();
    let second_missing = store.parse_json_path(&missing).unwrap_err();
    let first_missing_message = first_missing.to_string();
    assert!(std::error::Error::source(&first_missing).is_some());
    let super::JsonLoadError::Io(first_io) = first_missing else {
        panic!("missing JSON must retain its filesystem error");
    };
    let super::JsonLoadError::Io(second_io) = second_missing else {
        panic!("cached missing JSON must retain its filesystem error");
    };
    assert_eq!(first_io.kind(), ErrorKind::NotFound);
    assert_eq!(first_missing_message, first_io.to_string());
    assert!(Arc::ptr_eq(&first_io, &second_io));

    let syntax = store.parse_json_path(&malformed).unwrap_err();
    let syntax_message = syntax.to_string();
    assert!(std::error::Error::source(&syntax).is_some());
    let super::JsonLoadError::Syntax(error) = syntax else {
        panic!("malformed JSON must retain its syntax error");
    };
    assert_eq!(syntax_message, error.to_string());
    assert_ne!(
        error.to_string(),
        "EOF while parsing a value at line 1 column 0"
    );

    let super::JsonLoadError::Io(error) = store.parse_json_path(&unreadable).unwrap_err() else {
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

#[test]
fn concurrent_json_successes_have_exact_parse_and_cache_hit_metrics() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/workspace-symbol/package.json");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&manifest)));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let store = SourceStore::new_observed(inventory, Some(Arc::clone(&observer)));
    let barrier = Arc::new(Barrier::new(CONCURRENT_CALLERS));

    let values = std::thread::scope(|scope| {
        let handles = (0..CONCURRENT_CALLERS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let manifest = &manifest;
                let store = &store;
                scope.spawn(move || {
                    barrier.wait();
                    store.parse_json_path(manifest).unwrap()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(values.iter().all(|value| Arc::ptr_eq(value, &values[0])));
    assert_eq!(store.json_parse_count(), 1);
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], CONCURRENT_CALLERS as u64);
    assert_eq!(work["manifest.parses"], 1);
    assert_eq!(work["manifest.cache_hits"], (CONCURRENT_CALLERS - 1) as u64);
    assert_eq!(work.get("manifest.errors").copied().unwrap_or_default(), 0);
}

#[test]
fn concurrent_json_failures_have_exact_parse_and_cache_hit_metrics() {
    let malformed = fixture("alpha.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&malformed)));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let store = SourceStore::new_observed(inventory, Some(Arc::clone(&observer)));
    let barrier = Arc::new(Barrier::new(CONCURRENT_CALLERS));

    let errors = std::thread::scope(|scope| {
        let handles = (0..CONCURRENT_CALLERS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let malformed = &malformed;
                let store = &store;
                scope.spawn(move || {
                    barrier.wait();
                    store.parse_json_path(malformed).unwrap_err()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    let super::JsonLoadError::Syntax(first) = &errors[0] else {
        panic!("malformed JSON must retain its syntax error");
    };
    assert!(errors.iter().all(|error| {
        matches!(error, super::JsonLoadError::Syntax(current) if Arc::ptr_eq(current, first))
    }));
    assert_eq!(store.json_parse_count(), 1);
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], CONCURRENT_CALLERS as u64);
    assert_eq!(work["manifest.parses"], 1);
    assert_eq!(work["manifest.cache_hits"], (CONCURRENT_CALLERS - 1) as u64);
    assert_eq!(work["manifest.errors"], 1);
}
