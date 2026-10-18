use super::super::*;
use std::path::PathBuf;

#[path = "finalization_adjacency.rs"]
mod adjacency;
#[path = "finalization_append.rs"]
mod append;
#[path = "finalization_static.rs"]
mod static_assertions;

pub(super) fn p(path: &str) -> PathBuf {
    PathBuf::from(path)
}

pub(super) fn n(path: &str) -> NodeId {
    NodeId::file(p(path))
}

pub(super) fn with_materialized_vitest_cache(mut graph: DepGraph) -> DepGraph {
    graph = graph.with_vitest_setup_projects(vec![VitestSetupProject {
        config: None,
        scope: Some(".".to_string()),
        filter: crate::codebase::test_discovery::ProjectTestFilter::from_project_ref(
            &crate::integration_tests::types::ConfigProject {
                config: None,
                workspace: false,
                policy_name: None,
                runner_project_arg: None,
                scope: Some(".".to_string()),
                include: Vec::new(),
                exclude: Vec::new(),
                declared_deadlines: Default::default(),
                vitest_setup: Vec::new(),
            },
        )
        .unwrap(),
        tests: Vec::new(),
        setups: Vec::new(),
    }]);
    let _ = graph.dependents_of_node(&n("/repo/unused.ts"));
    assert!(graph.vitest_setup_edges_materialized());
    graph
}
