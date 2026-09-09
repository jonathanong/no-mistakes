use super::*;

#[test]
fn hoisted_helper_inside_arrow_body_keeps_dynamic_import_reachable() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/arrow-hoisted-helper.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(
        deps.iter()
            .any(|entry| { entry.node.as_file() == Some(root.join("src/called.mts").as_path()) })
    );
}

#[test]
fn nested_class_eager_imports_are_owned_by_the_reachable_enclosing_function() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/nested-class-eager.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(
        deps.iter()
            .any(|entry| entry.node.as_file() == Some(root.join("src/called.mts").as_path()))
    );
    assert!(
        !deps
            .iter()
            .any(|entry| entry.node.as_file() == Some(root.join("src/uncalled.mts").as_path()))
    );
}

#[test]
fn invoked_static_method_reaches_its_hoisted_helper_dynamic_import() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let source = root.join("src/static-run-hoisted-helper.mts");
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(source)],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<HashSet<_>>();

    assert!(paths.contains(root.join("src/called.mts").as_path()));
    assert!(!paths.contains(root.join("src/uncalled.mts").as_path()));
}
