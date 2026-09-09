use super::*;

#[test]
fn named_callback_arguments_keep_function_scoped_dynamic_imports() {
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
        &[NodeId::file(root.join("src/named-callback.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/called.mts").as_path())));
}

#[test]
fn static_getter_reads_keep_getter_imports_reachable() {
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
        &[NodeId::file(root.join("src/static-getter-read.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/static-getter-loaded.mts").as_path())
    }));
}

#[test]
fn object_getter_reads_keep_getter_imports_reachable() {
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
        &[NodeId::file(root.join("src/object-getter-read.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/object-getter-loaded.mts").as_path())
    }));
}

#[test]
fn reassigned_object_binding_drops_previous_getter_imports() {
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
        &[NodeId::file(root.join("src/object-getter-reassigned.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(!deps.iter().any(|entry| {
        entry.node.as_file()
            == Some(
                root.join("src/object-getter-reassigned-loaded.mts")
                    .as_path(),
            )
    }));
}

#[test]
fn wrapped_default_class_default_export_resolves_to_its_canonical_scope() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
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
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let source = root.join("src/wrapped-default-class.mts");
    let roots =
        graph.expand_call_roots(&[crate::codebase::dependencies::graph::CallRoot::Function {
            file: source.clone(),
            symbol: "default".to_string(),
        }]);

    assert!(roots.iter().any(|root| {
        matches!(
            root,
            NodeId::Symbol { file, symbol, callable_id: Some(_), .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "Wrapped"
        )
    }));
}
