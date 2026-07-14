use super::AnalysisDataset;
use std::sync::atomic::Ordering;

impl AnalysisDataset {
    fn parse_counts(&self) -> (usize, usize) {
        (
            self.config_parses.load(Ordering::Relaxed),
            self.tsconfig_parses.load(Ordering::Relaxed),
        )
    }
}
use std::path::{Path, PathBuf};

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
    let dataset = AnalysisDataset::new(&root);

    let first_config = dataset.config(Some(&config)).unwrap();
    let second_config = dataset.config(Some(&config)).unwrap();
    let first_tsconfig = dataset.tsconfig(Some(&tsconfig)).unwrap();
    let second_tsconfig = dataset.tsconfig(Some(&tsconfig)).unwrap();

    assert!(std::sync::Arc::ptr_eq(&first_config, &second_config));
    assert!(std::sync::Arc::ptr_eq(&first_tsconfig, &second_tsconfig));
    assert_eq!(dataset.parse_counts(), (1, 1));
    assert_eq!(dataset.sources_for(&root).physical_read_count(), 2);
}

#[test]
fn workspace_indexes_are_built_once_and_reused() {
    let root = repository_fixture("test-cases/codebase-analysis/large-graph-monorepo/fixture");
    let dataset = AnalysisDataset::new(&root);
    let first = dataset.workspace();
    let second = dataset.workspace();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert!(first.shares_indexes_with(&second));
}

#[test]
fn nested_roots_reuse_the_request_source_store() {
    let root = repository_fixture("test-cases/codebase-analysis/large-graph-monorepo/fixture");
    let dataset = AnalysisDataset::new(&root);

    let request_sources = dataset.sources_for(&root);
    let nested_sources = dataset.sources_for(&root.join("apps/api"));

    assert!(std::sync::Arc::ptr_eq(&request_sources, &nested_sources));
}

#[test]
fn config_parse_failures_are_memoized_exactly_once() {
    let root = repository_fixture("test-cases/impacted-checks/multi-framework");
    let invalid = root.join("invalid.no-mistakes.yml");
    let dataset = AnalysisDataset::new(&root);

    let first = dataset.config(Some(Path::new("invalid.no-mistakes.yml")));
    let second = dataset.config(Some(&invalid));

    assert!(first.is_err());
    assert!(second.is_err());
    assert_eq!(dataset.parse_counts(), (1, 0));
    assert_eq!(dataset.sources_for(&root).physical_read_count(), 1);
}
