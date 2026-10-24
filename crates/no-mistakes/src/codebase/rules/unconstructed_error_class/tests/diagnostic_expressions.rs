use super::*;

#[test]
fn computed_and_conditional_constructors_have_meaningful_diagnostic_labels() {
    let root = fixture().parent().unwrap().join("diagnostics/fixture");
    let findings = run_check(&root, None, None).unwrap();
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 2]
    );
    assert!(findings[0].message.contains("flag ? T1 : T2"));
    assert!(findings[1].message.contains("reg[name]"));
    for finding in findings {
        assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
        assert!(!finding.message.contains("<unknown>"));
        assert!(finding.message.contains("constructor expression `"));
    }
}

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
