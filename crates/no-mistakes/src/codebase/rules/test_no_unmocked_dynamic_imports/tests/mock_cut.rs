use super::*;

fn mock_cut_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-mock-cut/fixture",
    ))
}

#[test]
fn standalone_reachability_prunes_mocked_intermediaries() {
    let root = mock_cut_fixture();
    for (config_name, expected_target) in [
        (".no-mistakes-mocked.yml", None),
        (".no-mistakes-unmocked.yml", Some("src/leaf.mts")),
        (".no-mistakes-alternate.yml", Some("src/leaf.mts")),
        (".no-mistakes-direct.yml", Some("src/direct-entry.mts")),
        (".no-mistakes-mixed.yml", Some("src/leaf.mts")),
    ] {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(config_name))).unwrap();
        let findings = check(&root, &config, None).unwrap();
        assert_eq!(
            findings.len(),
            usize::from(expected_target.is_some()),
            "{config_name}: {findings:?}"
        );
        if let Some(target) = expected_target {
            assert_eq!(
                findings[0].target.as_deref(),
                Some(target),
                "{config_name}: {findings:?}"
            );
        }
    }
}

#[test]
fn reachable_dependency_cache_does_not_cross_mock_sets() {
    let root = mock_cut_fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let resolver = ImportResolver::new(&tsconfig);
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let test_file = root.join("tests/mocked.test.mts");
    let mocked = HashSet::from([root.join("src/replaced.mts")]);
    let unmocked = HashSet::new();
    for order in [[&mocked, &unmocked], [&unmocked, &mocked]] {
        let dependency_cache = DashMap::new();
        for mocks in order {
            let result = reachable::collect(
                reachable::ReachableContext {
                    root: &root,
                    config: &config,
                    resolver: &resolver,
                    graph: &graph,
                    graph_files: None,
                    file_universe: None,
                    shared: None,
                    file_cache: None,
                },
                &test_file,
                mocks,
                &dependency_cache,
            )
            .unwrap();
            assert_eq!(
                result.findings.len(),
                usize::from(mocks.is_empty()),
                "cache order must not change reachability for {mocks:?}"
            );
        }
    }
}
