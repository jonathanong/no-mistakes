use super::*;

fn fixture() -> std::path::PathBuf {
    root().with_file_name("call-reachability-fixture")
}

#[test]
fn closed_stage_methods_and_invoked_thunks_reach_only_their_proven_queries() {
    let findings = crate::codebase::rules::run_check(&fixture(), None, None).unwrap();
    let occurrences = findings
        .iter()
        .map(|finding| (finding.line, finding.import.as_deref().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(
        occurrences,
        [
            (11, "stage.run"),
            (15, "both"),
            (28, "objectStage.run"),
            (59, "stage.run"),
            (67, "stage.run"),
            (76, "stage.run"),
            (89, "stage.run")
        ],
        "{findings:#?}"
    );
    let stage = findings.iter().find(|finding| finding.line == 11).unwrap();
    assert!(stage.message.contains("q"), "{stage:?}");
    assert!(findings
        .iter()
        .find(|finding| finding.line == 15)
        .unwrap()
        .message
        .contains("both →"));
}

#[test]
fn stage_methods_are_shared_canonical_call_targets() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let stage = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.file == root.join("src/entry.mts") && site.line == 11)
        .unwrap();
    assert!(!stage.has_unknown_target(), "{stage:?}");
    assert_eq!(stage.target_nodes().count(), 2, "{stage:?}");
    for line in [33, 37, 41, 47, 55, 63, 64, 72, 80, 84] {
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| {
                site.file == stage.file && site.line == line && site.source_callee == "stage.run"
            })
            .unwrap();
        assert!(site.target_nodes().next().is_none(), "{site:?}");
    }
}
