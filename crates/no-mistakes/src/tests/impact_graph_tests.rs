use super::*;
use no_mistakes::codebase::dependencies::graph::{EdgeKind, NodeId, VitestSetupField};

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn filter(name: &str) -> TestFileFilter {
    let root = fixture(name);
    let visible = VisiblePathSnapshot::new(&root);
    let config = no_mistakes::config::v2::load_v2_config_from_visible(
        &root,
        None,
        &visible.paths_for(&root),
    )
    .unwrap();
    build_test_impact_graph(&root, None, &config, None, false)
        .unwrap()
        .test_filter
}

#[test]
fn impact_filter_preserves_configured_dotnet_test_projects() {
    let root = fixture("dotnet-test-plan");
    let filter = filter("dotnet-test-plan");
    assert!(filter.is_match(
        &root,
        &root.join("dotnet-clients/tests/App.Tests/FeedServiceTests.cs")
    ));
    assert!(!filter.is_match(&root, &root.join("dotnet-clients/src/App/FeedService.cs")));
}

#[test]
fn impact_filter_preserves_configured_swift_test_projects() {
    let root = fixture("swift-test-plan");
    let filter = filter("swift-test-plan");
    assert!(filter.is_match(
        &root,
        &root.join("swift-clients/core/Tests/ExampleCoreTests/APIClientTests.swift")
    ));
    assert!(!filter.is_match(
        &root,
        &root.join("swift-clients/core/Sources/ExampleCore/APIClient.swift")
    ));
}

#[test]
fn impact_graph_keeps_native_test_filters_when_optional_vitest_is_invalid() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/impact-invalid-vitest-config"),
    );
    let visible = VisiblePathSnapshot::new(&root);
    let config = no_mistakes::config::v2::load_v2_config_from_visible(
        &root,
        None,
        &visible.paths_for(&root),
    )
    .unwrap();
    let impact = build_test_impact_graph(&root, None, &config, None, false).unwrap();

    assert!(impact
        .test_filter
        .is_match(&root, &root.join("tests/ServiceTests.cs")));
    assert!(impact.vitest_discovered.tests.is_empty());
}

#[test]
fn impact_graph_keeps_valid_vitest_discovery_errors_strict() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/impact-invalid-vitest-discovery"),
    );
    let visible = VisiblePathSnapshot::new(&root);
    let config = no_mistakes::config::v2::load_v2_config_from_visible(
        &root,
        None,
        &visible.paths_for(&root),
    )
    .unwrap();

    assert!(build_test_impact_graph(&root, None, &config, None, false).is_err());
}

#[test]
fn impact_graph_connects_final_catalog_setup_alias_to_its_owner() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-project-tsconfig-setup"),
    );
    let config_path = root.join(".no-mistakes.yml");
    let visible = VisiblePathSnapshot::new(&root);
    let config = no_mistakes::config::v2::load_v2_config_from_visible(
        &root,
        Some(&config_path),
        &visible.paths_for(&root),
    )
    .unwrap();
    let impact = build_test_impact_graph(&root, None, &config, Some(&config_path), false).unwrap();

    assert_eq!(
        impact
            .graph
            .dependents_of_node(&NodeId::file(root.join("packages/unit/setup/aliased.ts"),)),
        Some(&vec![(
            NodeId::file(root.join("packages/unit/tests/owner.test.ts")),
            EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
        )])
    );
}
