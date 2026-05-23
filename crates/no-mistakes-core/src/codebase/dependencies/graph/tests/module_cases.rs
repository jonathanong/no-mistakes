use super::*;

#[test]
fn dynamic_imports_inside_uncalled_functions_are_pruned() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/entry.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps.iter().filter_map(|entry| entry.node.as_file()).collect();

    assert!(paths.contains(root.join("src/called.mts").as_path()));
    assert!(!paths.contains(root.join("src/uncalled.mts").as_path()));
}

#[test]
fn unknown_top_level_calls_keep_function_scoped_dynamic_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/unknown.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/uncalled.mts").as_path())));
}

#[test]
fn graph_includes_external_module_and_package_dependency_nodes() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-modules"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::all()).unwrap();

    let deps = graph.deps_of(&[NodeId::File(root.join("src/entry.mts"))], None, None);
    assert!(deps.iter().any(|entry| {
        entry.node == NodeId::Module("@react/client".to_string())
            && entry.via.contains(&EdgeKind::Import)
    }));

    let manifest_deps = graph.deps_of(&[NodeId::File(root.join("package.json"))], None, None);
    assert!(manifest_deps.iter().any(|entry| {
        entry.node == NodeId::Module("@react/server".to_string())
            && entry.via.contains(&EdgeKind::PackageDependency)
    }));
}

#[test]
fn pkg_name_scoped_no_subpath() {
    assert_eq!(package_name_from_spec("@x/api"), "@x/api");
}

#[test]
fn pkg_name_scoped_with_subpath() {
    assert_eq!(package_name_from_spec("@x/api/utils"), "@x/api");
}

#[test]
fn pkg_name_unscoped_no_subpath() {
    assert_eq!(package_name_from_spec("lodash"), "lodash");
}

#[test]
fn pkg_name_unscoped_with_subpath() {
    assert_eq!(package_name_from_spec("lodash/merge"), "lodash");
}
