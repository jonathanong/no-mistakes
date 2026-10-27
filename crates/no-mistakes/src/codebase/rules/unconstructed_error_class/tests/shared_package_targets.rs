use super::*;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

#[test]
fn aliases_captured_after_package_member_mutation_keep_constructor_identity() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("post-mutation-aliases/fixture");
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: true,
            extends: true,
            ..Default::default()
        },
    )
    .unwrap();
    for callee in ["Snapshot", "pg.Pool"] {
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| site.file.ends_with("app/facade.ts") && site.source_callee == callee)
            .unwrap();
        assert_eq!(
            graph.constructor_class(&site.file, site.offset),
            None,
            "{callee}"
        );
    }
}
