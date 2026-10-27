use super::*;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

#[test]
fn legacy_graphs_without_source_metadata_use_a_descriptive_constructor_fallback() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
    use crate::codebase::ts_resolver::TsConfig;
    let root = fixture().parent().unwrap().join("diagnostics/fixture");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let application = config
        .rule_applications(RULE_ID)
        .into_iter()
        .next()
        .unwrap();
    let graph = DepGraph::build_with_plan(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap()
    .without_class_ancestry_metadata();
    // Erase only the optional metadata at the graph compatibility boundary;
    // source call targets and the internal unknown-callee sentinel stay intact.
    let findings = super::super::diagnostics::unknown_constructs(
        &root,
        application,
        &graph,
        |_| false,
        |_| true,
    );
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().all(|finding| finding
        .message
        .contains("computed or conditional constructor expression")
        && !finding.message.contains("<unknown>")));
}
