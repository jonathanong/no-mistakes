use super::*;

fn dynamic_import_deps(file: &str) -> HashSet<std::path::PathBuf> {
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
    graph
        .deps_of(
            &[NodeId::file(root.join(file))],
            None,
            Some(&[EdgeKind::DynamicImport].into()),
        )
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(PathBuf::from))
        .collect()
}

#[test]
fn constructor_this_member_keeps_instance_method_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let deps = dynamic_import_deps("src/this-member-local.mts");
    assert!(deps.contains(root.join("src/this-member-loaded.mts").as_path()));
}

#[test]
fn static_this_member_keeps_static_method_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let deps = dynamic_import_deps("src/this-member-static.mts");
    assert!(deps.contains(root.join("src/this-member-loaded.mts").as_path()));
}

#[test]
fn derived_this_member_resolves_to_the_inherited_method() {
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
    let source = root.join("src/this-member-inherited.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: source.clone(),
        symbol: "Derived/run".to_string(),
    }]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(traces.iter().any(|trace| matches!(
        &trace.target,
        NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == source.as_path() && symbol.as_ref() == "Base/load"
    )));
}

#[test]
fn derived_this_member_resolves_to_the_override() {
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
    let source = root.join("src/this-member-derived.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: source.clone(),
        symbol: "Derived/run".to_string(),
    }]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(traces.iter().any(|trace| matches!(
        &trace.target,
        NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == source.as_path() && symbol.as_ref() == "Derived/load"
    )));
    assert!(!traces.iter().any(|trace| matches!(
        &trace.target,
        NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == source.as_path() && symbol.as_ref() == "Base/load"
    )));
}

#[test]
fn instance_this_member_call_resolves_to_the_method() {
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
    let source = root.join("src/this-member-method.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: source.clone(),
        symbol: "Service/run".to_string(),
    }]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(traces.iter().any(|trace| matches!(
        &trace.target,
        NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == source.as_path() && symbol.as_ref() == "Service/load"
    )));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == source
            && site.source_callee == "this.load"
            && matches!(
                &site.target,
                ResolvedCallTarget::RepositoryFunction { scope, .. } if scope == "Service/load"
            )
    }));
}

#[test]
fn computed_this_member_stays_unresolved() {
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
    let source = root.join("src/this-member-computed.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: source.clone(),
        symbol: "Service/run".to_string(),
    }]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(!traces.iter().any(|trace| matches!(
        &trace.target,
        NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == source.as_path() && symbol.as_ref() == "Service/load"
    )));
}
