use super::AnalysisDataset;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn dataset(root: &Path) -> AnalysisDataset {
    AnalysisDataset::new_observed(root, None)
}

fn observed_dataset(root: &Path) -> (AnalysisDataset, Arc<crate::diagnostics::InvocationObserver>) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    (
        AnalysisDataset::new_observed(root, Some(Arc::clone(&observer))),
        observer,
    )
}

fn repository_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn config_and_tsconfig_parses_are_memoized_exactly_once() {
    let root =
        repository_fixture("test-cases/codebase-analysis/playwright-config-path-graph/fixture");
    let config = root.join("custom.no-mistakes.yml");
    let tsconfig = repository_fixture(
        "fixtures/napi/analyze-project-dynamic-import-reachability/tsconfig.json",
    );
    let (dataset, observer) = observed_dataset(&root);

    let first_config = dataset.config(Some(&config)).unwrap();
    let second_config = dataset.config(Some(&config)).unwrap();
    let first_tsconfig = dataset.tsconfig(Some(&tsconfig)).unwrap();
    let second_tsconfig = dataset.tsconfig(Some(&tsconfig)).unwrap();

    assert!(std::sync::Arc::ptr_eq(&first_config, &second_config));
    assert!(std::sync::Arc::ptr_eq(&first_tsconfig, &second_tsconfig));
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 4);
    assert_eq!(work["manifest.parses"], 2);
    assert_eq!(work["manifest.cache_hits"], 2);
    assert_eq!(dataset.sources_for(&root).physical_read_count(), 2);
}

#[test]
fn workspace_indexes_are_built_once_and_reused() {
    let root = repository_fixture("test-cases/codebase-analysis/large-graph-monorepo/fixture");
    let dataset = dataset(&root);
    let first = dataset.workspace();
    let second = dataset.workspace();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert!(first.shares_indexes_with(&second));
}

#[test]
fn nested_roots_reuse_the_request_source_store() {
    let root = repository_fixture("test-cases/codebase-analysis/large-graph-monorepo/fixture");
    let dataset = dataset(&root);

    let request_sources = dataset.sources_for(&root);
    let nested_sources = dataset.sources_for(&root.join("apps/api"));

    assert!(std::sync::Arc::ptr_eq(&request_sources, &nested_sources));
}

#[test]
fn config_parse_failures_are_memoized_exactly_once() {
    let root = repository_fixture("test-cases/impacted-checks/multi-framework");
    let invalid = root.join("invalid.no-mistakes.yml");
    let (dataset, observer) = observed_dataset(&root);

    let first = dataset.config(Some(Path::new("invalid.no-mistakes.yml")));
    let second = dataset.config(Some(&invalid));

    assert!(first.is_err());
    assert!(second.is_err());
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 2);
    assert_eq!(work["manifest.parses"], 1);
    assert_eq!(work["manifest.cache_hits"], 1);
    assert_eq!(work["manifest.errors"], 1);
    assert_eq!(dataset.sources_for(&root).physical_read_count(), 1);
}

#[test]
fn automatic_and_explicit_manifest_paths_share_cache_entries() {
    let root =
        repository_fixture("test-cases/codebase-analysis/forbidden-dependencies-passes/fixture");
    let (dataset, observer) = observed_dataset(&root);

    let automatic_config = dataset.config(None).unwrap();
    let explicit_config = dataset.config(Some(Path::new(".no-mistakes.yml"))).unwrap();
    let automatic_tsconfig = dataset.tsconfig(None).unwrap();
    let explicit_tsconfig = dataset.tsconfig(Some(Path::new("tsconfig.json"))).unwrap();

    assert!(Arc::ptr_eq(&automatic_config, &explicit_config));
    assert!(Arc::ptr_eq(&automatic_tsconfig, &explicit_tsconfig));
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 4);
    assert_eq!(work["manifest.parses"], 2);
    assert_eq!(work["manifest.cache_hits"], 2);
}

#[test]
fn same_root_distinct_manifest_paths_remain_isolated() {
    let root = repository_fixture("fixtures/gitignore/integration-aggregate");
    let explicit_config = root.join("explicit.no-mistakes.yml");
    let external_tsconfig = repository_fixture(
        "fixtures/napi/analyze-project-dynamic-import-reachability/tsconfig.json",
    );
    let (dataset, observer) = observed_dataset(&root);

    let automatic_config = dataset.config(None).unwrap();
    let explicit_config = dataset.config(Some(&explicit_config)).unwrap();
    let automatic_tsconfig = dataset.tsconfig(None).unwrap();
    let external_tsconfig = dataset.tsconfig(Some(&external_tsconfig)).unwrap();

    assert!(!Arc::ptr_eq(&automatic_config, &explicit_config));
    assert!(!Arc::ptr_eq(&automatic_tsconfig, &external_tsconfig));
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 4);
    assert_eq!(work["manifest.parses"], 4);
    assert_eq!(work.get("manifest.cache_hits"), None);
}

#[test]
fn concurrent_same_manifest_requests_parse_once_and_count_waiters_as_hits() {
    use rayon::prelude::*;

    let root =
        repository_fixture("test-cases/codebase-analysis/playwright-config-path-graph/fixture");
    let config = root.join("custom.no-mistakes.yml");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let dataset = Arc::new(AnalysisDataset::new_observed(
        &root,
        Some(Arc::clone(&observer)),
    ));

    let configs = (0..16)
        .into_par_iter()
        .map(|_| dataset.config(Some(&config)).unwrap())
        .collect::<Vec<_>>();

    assert!(configs
        .iter()
        .all(|config| Arc::ptr_eq(config, &configs[0])));
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 16);
    assert_eq!(work["manifest.parses"], 1);
    assert_eq!(work["manifest.cache_hits"], 15);
}
