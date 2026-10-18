use super::super::SourceStore;
use super::YamlLoadError;
use crate::codebase::ts_source::FileInventory;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};

const CONCURRENT_CALLERS: usize = 16;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/analysis-dataset/source-store")
        .join(path)
}

fn store(path: &Path) -> SourceStore {
    SourceStore::new(Arc::new(FileInventory::from_paths(&[path.to_path_buf()])))
}

#[test]
fn concurrent_yaml_successes_share_one_parse_and_normalized_path_slot() {
    let path = fixture("valid.yml");
    let alias = path.parent().unwrap().join(".").join("valid.yml");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = SourceStore::new_observed(inventory, Some(Arc::clone(&observer)));
    let barrier = Arc::new(Barrier::new(CONCURRENT_CALLERS));

    let documents = std::thread::scope(|scope| {
        let handles = (0..CONCURRENT_CALLERS)
            .map(|index| {
                let barrier = Arc::clone(&barrier);
                let path = if index % 2 == 0 { &path } else { &alias };
                let sources = &sources;
                scope.spawn(move || {
                    barrier.wait();
                    sources.parse_yaml_path(path).unwrap()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(documents
        .iter()
        .all(|document| Arc::ptr_eq(document, &documents[0])));
    assert_eq!(documents[0]["name"].as_str(), Some("yaml source"));
    assert_eq!(sources.yaml_parse_count(), 1);
    assert_eq!(sources.physical_read_count(), 1);
    let work = observer.snapshot().work;
    assert_eq!(work["yaml.requests"], CONCURRENT_CALLERS as u64);
    assert_eq!(work["yaml.parses"], 1);
    assert_eq!(work["yaml.cache_hits"], (CONCURRENT_CALLERS - 1) as u64);
    assert_eq!(work.get("yaml.errors").copied().unwrap_or_default(), 0);
}

#[test]
fn yaml_read_failures_retain_and_cache_the_original_io_error() {
    let missing = fixture("missing.yml");
    let sources = store(&missing);

    let first = sources.parse_yaml_path(&missing).unwrap_err();
    let second = sources.parse_yaml_path(&missing).unwrap_err();
    assert!(first.is_io());
    assert!(std::error::Error::source(&first).is_some());
    let YamlLoadError::Io(first_io) = first else {
        panic!("missing YAML must retain its filesystem error");
    };
    let YamlLoadError::Io(second_io) = second else {
        panic!("cached missing YAML must retain its filesystem error");
    };
    assert_eq!(first_io.kind(), ErrorKind::NotFound);
    assert!(Arc::ptr_eq(&first_io, &second_io));
    assert_eq!(sources.yaml_parse_count(), 1);
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn yaml_syntax_failures_retain_and_cache_the_original_parser_error() {
    let malformed = fixture("malformed.yml");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(std::slice::from_ref(&malformed))),
        Some(Arc::clone(&observer)),
    );
    let barrier = Arc::new(Barrier::new(CONCURRENT_CALLERS));
    let errors = std::thread::scope(|scope| {
        let handles = (0..CONCURRENT_CALLERS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let malformed = &malformed;
                let sources = &sources;
                scope.spawn(move || {
                    barrier.wait();
                    sources.parse_yaml_path(malformed).unwrap_err()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    let message = errors[0].to_string();
    assert!(!errors[0].is_io());
    assert!(std::error::Error::source(&errors[0]).is_some());
    let YamlLoadError::Syntax(first_syntax) = &errors[0] else {
        panic!("malformed YAML must retain its parser error");
    };
    assert_eq!(message, first_syntax.to_string());
    assert!(errors.iter().all(|error| {
        matches!(error, YamlLoadError::Syntax(current) if Arc::ptr_eq(current, first_syntax))
    }));
    assert_eq!(sources.yaml_parse_count(), 1);
    assert_eq!(sources.physical_read_count(), 1);
    let work = observer.snapshot().work;
    assert_eq!(work["yaml.requests"], CONCURRENT_CALLERS as u64);
    assert_eq!(work["yaml.parses"], 1);
    assert_eq!(work["yaml.cache_hits"], (CONCURRENT_CALLERS - 1) as u64);
    assert_eq!(work["yaml.errors"], 1);
}

#[test]
fn parse_counters_are_request_scoped_and_json_counters_stay_independent() {
    let yaml = fixture("valid.yml");
    let first_observer = crate::diagnostics::InvocationObserver::new(true);
    let first = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(std::slice::from_ref(&yaml))),
        Some(Arc::clone(&first_observer)),
    );
    assert!(
        first.yaml_parses.get().is_none(),
        "non-YAML requests allocate no YAML slots"
    );
    first.parse_yaml_path(&yaml).unwrap();
    assert!(first.yaml_parses.get().is_some());
    assert_eq!(first.yaml_parse_count(), 1);
    assert_eq!(first.json_parse_count(), 0);

    let second_observer = crate::diagnostics::InvocationObserver::new(true);
    let second = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(std::slice::from_ref(&yaml))),
        Some(Arc::clone(&second_observer)),
    );
    second.parse_yaml_path(&yaml).unwrap();
    assert_eq!(second.yaml_parse_count(), 1);
    assert_eq!(second.json_parse_count(), 0);
    assert_eq!(first_observer.snapshot().work["yaml.parses"], 1);
    assert_eq!(second_observer.snapshot().work["yaml.parses"], 1);

    let json = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/workspace-symbol/package.json");
    let json_store = store(&json);
    json_store.parse_json_path(&json).unwrap();
    assert_eq!(json_store.json_parse_count(), 1);
    assert_eq!(json_store.yaml_parse_count(), 0);
}
