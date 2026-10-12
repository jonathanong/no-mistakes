use super::*;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};

fn fixture_lockfile(name: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/forbidden-workspace-closure")
        .join(name)
        .join("pnpm-lock.yaml");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    (path, content)
}

fn malformed_lockfile() -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/pnpm12-issue-1035/malformed/pnpm-lock.yaml");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    (path, content)
}

#[test]
fn cache_constructs_once_for_concurrent_calls_and_isolates_paths() {
    let (first_path, first_content) = fixture_lockfile("lockfile-dependency-types");
    let (second_path, second_content) = fixture_lockfile("lockfile-exact-workspace-range");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let cache = Arc::new(LockfileImporterCache::with_observer(Some(observer.clone())));
    let start = Arc::new(Barrier::new(8));

    std::thread::scope(|scope| {
        let mut requests = Vec::new();
        for _ in 0..8 {
            let cache = Arc::clone(&cache);
            let path = first_path.clone();
            let content = first_content.clone();
            let start = Arc::clone(&start);
            requests.push(scope.spawn(move || {
                start.wait();
                cache.get_or_compute(&path, || parse_importer_content(&content))
            }));
        }
        for request in requests {
            assert!(request.join().unwrap().is_ok());
        }
    });
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.lockfile_importer_constructions"],
        1
    );

    let second = cache.get_or_compute(&second_path, || parse_importer_content(&second_content));
    assert!(second.is_ok());
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.lockfile_importer_constructions"],
        2
    );
}

#[test]
fn cache_retains_malformed_and_read_error_results() {
    let (malformed_path, malformed_content) = malformed_lockfile();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let cache = LockfileImporterCache::with_observer(Some(observer.clone()));
    let malformed = cache.get_or_compute(&malformed_path, || {
        parse_importer_content(&malformed_content)
    });
    let cached_malformed = cache.get_or_compute(&malformed_path, || {
        panic!("malformed result should already be cached")
    });
    assert!(matches!(
        malformed.as_ref().as_ref(),
        Ok(ParsedImporters::Malformed)
    ));
    assert!(Arc::ptr_eq(&malformed, &cached_malformed));
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.lockfile_importer_constructions"],
        1
    );

    let missing_path = malformed_path.with_file_name("missing-lockfile.yaml");
    let read_error = cache.get_or_compute(&missing_path, || Err("fixture read error".to_string()));
    let cached_read_error = cache.get_or_compute(&missing_path, || {
        panic!("read error should already be cached")
    });
    assert!(matches!(
        read_error.as_ref().as_ref(),
        Err(error) if error == "fixture read error"
    ));
    assert!(Arc::ptr_eq(&read_error, &cached_read_error));
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.lockfile_importer_constructions"],
        2
    );
}
