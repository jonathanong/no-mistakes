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
        first.yaml_documents.get().is_none(),
        "non-YAML requests allocate no YAML slots"
    );
    first.parse_yaml_path(&yaml).unwrap();
    assert!(first.yaml_documents.get().is_some());
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

#[test]
fn yaml_documents_keep_parsed_documents_and_cache_one_read() {
    let multi = fixture("multi.yml");
    let sources = store(&multi);
    let first = sources.parse_yaml_documents_path(&multi).unwrap();
    let second = sources.parse_yaml_documents_path(&multi).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.len(), 2);
    assert_eq!(first[0]["name"].as_str(), Some("first"));
    assert_eq!(first[1]["name"].as_str(), Some("second"));
    assert_eq!(sources.yaml_parse_count(), 1);
    assert_eq!(sources.physical_read_count(), 1);

    let partial = fixture("partial.yml");
    let kept = store(&partial).parse_yaml_documents_path(&partial).unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0]["name"].as_str(), Some("kept"));

    let comments = fixture("comments.yml");
    let only_comments = store(&comments)
        .parse_yaml_documents_path(&comments)
        .unwrap();
    assert!(
        only_comments.is_empty() || only_comments.iter().all(|document| !document.is_mapping()),
        "{only_comments:?}"
    );

    let malformed = fixture("malformed.yml");
    let syntax = store(&malformed)
        .parse_yaml_documents_path(&malformed)
        .unwrap_err();
    assert!(!syntax.is_io());

    let missing = fixture("missing.yml");
    let io_error = store(&missing)
        .parse_yaml_documents_path(&missing)
        .unwrap_err();
    assert!(io_error.is_io());
}

#[test]
fn yaml_views_share_one_parse() {
    let path = fixture("valid.yml");
    let sources = store(&path);
    let single = sources.parse_yaml_path(&path).unwrap();
    let documents = sources.parse_yaml_documents_path(&path).unwrap();
    assert_eq!(documents.len(), 1);
    assert!(Arc::ptr_eq(&single, &documents[0]));
    assert_eq!(single["name"].as_str(), Some("yaml source"));
    assert_eq!(sources.yaml_parse_count(), 1);
    assert_eq!(sources.physical_read_count(), 1);

    let multi = fixture("multi.yml");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let multi_store = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(std::slice::from_ref(&multi))),
        Some(Arc::clone(&observer)),
    );
    let docs = multi_store.parse_yaml_documents_path(&multi).unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("yaml.errors")
            .copied()
            .unwrap_or(0),
        0
    );
    let strict = multi_store.parse_yaml_path(&multi).unwrap_err();
    assert!(!strict.is_io());
    let raw = std::fs::read_to_string(&multi).unwrap();
    let expected = serde_yaml::from_str::<serde_yaml::Value>(&raw).unwrap_err();
    assert_eq!(strict.to_string(), expected.to_string());
    let again = multi_store.parse_yaml_path(&multi).unwrap_err();
    assert_eq!(again.to_string(), strict.to_string());
    assert_eq!(multi_store.yaml_parse_count(), 1);
    assert_eq!(multi_store.physical_read_count(), 1);
    let work = observer.snapshot().work;
    assert_eq!(work["yaml.parses"], 1);
    assert_eq!(work["yaml.errors"], 1);
    assert_eq!(work["yaml.requests"], 3);
    assert_eq!(work["yaml.cache_hits"], 2);

    let partial = fixture("partial.yml");
    let partial_store = store(&partial);
    let partial_strict = partial_store.parse_yaml_path(&partial).unwrap_err();
    let partial_raw = std::fs::read_to_string(&partial).unwrap();
    let partial_expected = serde_yaml::from_str::<serde_yaml::Value>(&partial_raw).unwrap_err();
    assert_eq!(partial_strict.to_string(), partial_expected.to_string());
    let kept = partial_store.parse_yaml_documents_path(&partial).unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0]["name"].as_str(), Some("kept"));
    assert_eq!(partial_store.yaml_parse_count(), 1);

    let comments = fixture("comments.yml");
    let comments_store = store(&comments);
    let comments_strict = comments_store.parse_yaml_path(&comments).unwrap();
    assert!(comments_strict.is_null());
    let only_comments = comments_store.parse_yaml_documents_path(&comments).unwrap();
    assert_eq!(only_comments.len(), 1);
    assert!(Arc::ptr_eq(&comments_strict, &only_comments[0]));
    assert_eq!(comments_store.yaml_parse_count(), 1);

    let empty = fixture("empty.yml");
    let empty_store = store(&empty);
    let empty_strict = empty_store.parse_yaml_path(&empty).unwrap();
    assert!(empty_strict.is_null());
    let empty_docs = empty_store.parse_yaml_documents_path(&empty).unwrap();
    assert_eq!(empty_docs.len(), 1);
    assert!(Arc::ptr_eq(&empty_strict, &empty_docs[0]));
    assert_eq!(empty_store.yaml_parse_count(), 1);
}

#[test]
fn strict_view_of_no_documents_is_null() {
    let split = super::SplitYaml {
        documents: Vec::new(),
        failure: None,
    };
    let strict = super::single_view(&split).unwrap();
    assert!(strict.is_null());
    let documents = super::document_view(&split).unwrap();
    assert!(documents.is_empty());
}
