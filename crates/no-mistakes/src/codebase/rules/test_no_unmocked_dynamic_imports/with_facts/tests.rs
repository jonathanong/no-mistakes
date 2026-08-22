use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::ts_resolver::{ScopedImportResolver, TsConfigCatalog};
use std::collections::HashMap;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports/fixture"),
    )
}

fn tsconfig(root: &Path) -> TsConfig {
    TsConfig {
        dir: root.to_path_buf(),
        paths: Vec::new(),
        paths_dir: root.to_path_buf(),
        base_url: None,
    }
}

fn dynamic_facts(path: &Path, source: &str) -> std::sync::Arc<CheckFileFacts> {
    CheckFileFacts {
        source: Some(source.into()),
        dynamic_imports: Some(super::super::ast::extract(path, source).unwrap()),
        ..Default::default()
    }
    .into()
}

#[test]
fn per_test_requires_a_prepared_test_fact_entry() {
    let root = fixture();
    let test = root.join("tests/bad.test.mts");
    let visible: crate::fx::PathSet = [test.clone()].into_iter().collect();
    let config = NoMistakesConfig::default();
    let tsconfig = tsconfig(&root);
    let catalog = TsConfigCatalog::forced(&root, tsconfig, None);
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    let graph = from_raw_maps(root.clone(), Default::default(), Default::default());
    let graph_files = GraphFiles::from_files(vec![test.clone()]);
    let shared = CheckFactMap {
        files: vec![test.clone()],
        ..Default::default()
    };
    let dependency_cache = DashMap::new();

    let error = per_test::analyze(
        per_test::Request {
            root: &root,
            config: &config,
            resolver: &resolver,
            graph: &graph,
            graph_files: &graph_files,
            visible_files: &visible,
            manual_mocks: &HashSet::new(),
            setup_data: &[],
            shared: &shared,
            dependency_cache: &dependency_cache,
            defer_suppression: false,
        },
        test,
    )
    .err()
    .expect("missing test facts must be reported");

    assert!(error.to_string().contains("missing shared facts"));
}

#[test]
fn per_test_analyzes_empty_prepared_dynamic_facts() {
    let root = fixture();
    let test = root.join("tests/bad.test.mts");
    let visible: crate::fx::PathSet = [test.clone()].into_iter().collect();
    let config = NoMistakesConfig::default();
    let tsconfig = tsconfig(&root);
    let catalog = TsConfigCatalog::forced(&root, tsconfig, None);
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    let graph = from_raw_maps(root.clone(), Default::default(), Default::default());
    let graph_files = GraphFiles::from_files(vec![test.clone()]);
    let mut shared = CheckFactMap {
        files: vec![test.clone()],
        ..Default::default()
    };
    shared.ts.insert(
        test.clone(),
        dynamic_facts(&test, "test('prepared facts', () => {});"),
    );
    let dependency_cache = DashMap::new();

    let result = per_test::analyze(
        per_test::Request {
            root: &root,
            config: &config,
            resolver: &resolver,
            graph: &graph,
            graph_files: &graph_files,
            visible_files: &visible,
            manual_mocks: &HashSet::new(),
            setup_data: &[],
            shared: &shared,
            dependency_cache: &dependency_cache,
            defer_suppression: false,
        },
        test,
    )
    .unwrap();

    assert!(result.direct_findings.is_empty());
    assert!(result.reachable_findings.is_empty());
    assert!(result.reachable_suppression_file.is_none());
}

#[test]
fn prepared_reachability_skips_unavailable_facts_and_keeps_disabled_origin() {
    let root = fixture();
    let test = root.join("tests/disabled.test.mts");
    let dependency = root.join("src/unmocked-next-dynamic-component.mts");
    let visible: crate::fx::PathSet = [test.clone(), dependency.clone()].into_iter().collect();
    let config = NoMistakesConfig::default();
    let tsconfig = tsconfig(&root);
    let catalog = TsConfigCatalog::forced(&root, tsconfig, None);
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    let mut forward = HashMap::new();
    forward.insert(test.clone(), vec![dependency.clone()]);
    let graph = from_raw_maps(root.clone(), forward, Default::default());
    let graph_files = GraphFiles::from_files(vec![test.clone(), dependency.clone()]);
    let dependency_cache = DashMap::new();

    for facts in [
        None,
        Some(
            CheckFileFacts {
                parse_error: Some("fixture parse error".to_string()),
                ..Default::default()
            }
            .into(),
        ),
        Some(
            CheckFileFacts {
                source: Some("export const helper = true;".into()),
                ..Default::default()
            }
            .into(),
        ),
    ] {
        let mut shared = CheckFactMap {
            files: vec![test.clone(), dependency.clone()],
            ..Default::default()
        };
        shared.ts.insert(
            test.clone(),
            dynamic_facts(
                &test,
                "// no-mistakes-disable-file test-no-unmocked-dynamic-imports\ntest('disabled', () => {});",
            ),
        );
        if let Some(facts) = facts {
            shared.ts.insert(dependency.clone(), facts);
        }

        let result = per_test::analyze(
            per_test::Request {
                root: &root,
                config: &config,
                resolver: &resolver,
                graph: &graph,
                graph_files: &graph_files,
                visible_files: &visible,
                manual_mocks: &HashSet::new(),
                setup_data: &[],
                shared: &shared,
                dependency_cache: &dependency_cache,
                defer_suppression: true,
            },
            test.clone(),
        )
        .unwrap();

        assert!(result.reachable_findings.is_empty());
        assert_eq!(
            result.reachable_suppression_file.as_deref(),
            Some("tests/disabled.test.mts")
        );
    }
}
