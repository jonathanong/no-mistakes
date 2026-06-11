use super::*;

#[test]
fn graph_config_options_for_plan_returns_none_without_config_driven_edges() {
    let explicit =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan::imports_and_workspace()
    )
    .is_none());

    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan {
            routes: true,
            ..GraphBuildPlan::default()
        }
    )
    .is_some());
    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan {
            queues: true,
            ..GraphBuildPlan::default()
        }
    )
    .is_some());
    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan {
            tests: true,
            ..GraphBuildPlan::default()
        }
    )
    .is_some());
    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan {
            http: true,
            ..GraphBuildPlan::default()
        }
    )
    .is_some());
}

#[test]
fn test_only_graph_build_applies_project_test_filters() {
    let root = fixture("test-plan-project-discovery");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            tests: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let testof_filter: std::collections::HashSet<EdgeKind> = [EdgeKind::TestOf].into();
    let ignored_source = root.join("web/storybook/skip/ignored.tsx");
    let ignored_test = root.join("web/storybook/skip/ignored.test.tsx");

    let dependents =
        graph.dependents_of(&[NodeId::File(ignored_source)], None, Some(&testof_filter));

    assert!(
        !dependents
            .iter()
            .any(|entry| entry.node.as_file() == Some(ignored_test.as_path())),
        "excluded project tests should not produce TestOf edges in test-only graph builds"
    );
}

#[test]
fn graph_build_with_explicit_config_overrides_discovered_test_filters() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("tests-impact-symbol"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let plan = GraphBuildPlan {
        tests: true,
        ..GraphBuildPlan::default()
    };
    let testof_filter: std::collections::HashSet<EdgeKind> = [EdgeKind::TestOf].into();
    let source = root.join("other.mts");
    let test = root.join("other.test.mts");

    let default_graph = DepGraph::build_with_plan(&root, &tsconfig, plan).unwrap();
    let explicit_graph = DepGraph::build_with_plan_and_config(
        &root,
        &tsconfig,
        plan,
        Some(Path::new("exclude-other-test.no-mistakes.yml")),
    )
    .unwrap();

    let default_dependents =
        default_graph.dependents_of(&[NodeId::File(source.clone())], None, Some(&testof_filter));
    let explicit_dependents =
        explicit_graph.dependents_of(&[NodeId::File(source)], None, Some(&testof_filter));

    assert!(default_dependents
        .iter()
        .any(|entry| entry.node.as_file() == Some(test.as_path())));
    assert!(!explicit_dependents
        .iter()
        .any(|entry| entry.node.as_file() == Some(test.as_path())));
}
