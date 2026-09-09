use super::*;

#[test]
fn import_reachability_resolves_this_member_through_the_class_index() {
    let reachability = include_str!("../../edge_import_reachability_traversal.rs");
    assert!(
        reachability.contains(".resolve_this_member("),
        "import reachability must resolve this.member through the class index",
    );
}

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

#[test]
fn mixed_instance_and_static_run_keep_this_member_kind() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let source = root.join("src/this-member-mixed-run.mts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&source),
        TsFactPlan {
            function_calls: true,
            ..TsFactPlan::default()
        },
    );
    let file_facts = facts.get(&source).expect("mixed-run fixture facts");
    let static_run = file_facts
        .class_member_callable_ids
        .iter()
        .find(|(_, name, _)| name == "run")
        .map(|(_, _, id)| *id)
        .expect("static run");
    let static_load = file_facts
        .class_member_callable_ids
        .iter()
        .find(|(_, name, _)| name == "load")
        .map(|(_, _, id)| *id)
        .expect("static load");
    let instance_run = file_facts
        .function_calls
        .iter()
        .find(|call| call.callee == "this.load" && call.caller_id != Some(static_run))
        .and_then(|call| call.caller_id)
        .expect("instance run");
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
    for (caller, expect_static) in [(instance_run, false), (static_run, true)] {
        let traces = graph.call_traces(
            &[NodeId::callable(&source, "Service/run", caller)],
            CallTraversal::Direct,
            None,
        );
        let load = traces.iter().find_map(|trace| match &trace.target {
            NodeId::Symbol {
                file,
                symbol,
                callable_id,
                ..
            } if file.as_ref() == source.as_path() && symbol.as_ref() == "Service/load" => {
                *callable_id
            }
            _ => None,
        });
        if expect_static {
            assert_eq!(load, Some(static_load));
        } else {
            assert_ne!(load, Some(static_load));
            assert!(load.is_some());
        }
    }
}

fn this_member_facts(file: &str) -> (std::path::PathBuf, crate::codebase::ts_source::facts::TsFileFacts) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let source = root.join(file);
    let facts = collect_ts_facts(
        std::slice::from_ref(&source),
        TsFactPlan {
            function_calls: true,
            ..TsFactPlan::default()
        },
    );
    let file_facts = facts.get(&source).cloned().expect("this-member fixture facts");
    (source, file_facts)
}

#[test]
fn this_member_index_covers_caller_id_fallbacks_and_invalid_callees() {
    let (_, method) = this_member_facts("src/this-member-method.mts");
    let method = CallableFileIndex::from_facts(&method);
    assert!(method
        .resolve_this_member(
            Some("Service/run"),
            None,
            "this.load",
            InvocationKind::Construct,
        )
        .is_none());
    assert!(method
        .resolve_this_member(Some("Service/run"), None, "this.foo.bar", InvocationKind::Call)
        .is_none());
    assert!(method
        .resolve_this_member(Some("Service/run"), None, "this.", InvocationKind::Call)
        .is_none());
    assert_eq!(
        method
            .resolve_this_member(Some("Service/run"), None, "this.load", InvocationKind::Call)
            .map(|resolved| resolved.callee),
        Some("Service/load".to_string()),
    );

    let (_, mixed) = this_member_facts("src/this-member-mixed-run.mts");
    let mixed = CallableFileIndex::from_facts(&mixed);
    assert!(mixed
        .resolve_this_member(Some("Service/run"), None, "this.load", InvocationKind::Call)
        .is_none());
}

#[test]
fn nested_this_member_resolves_from_unique_instance_and_static_methods() {
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
    let source = root.join("src/this-member-nested.mts");
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == source
            && site.source_callee == "this.load"
            && matches!(
                &site.target,
                ResolvedCallTarget::RepositoryFunction { scope, .. } if scope == "Service/load"
            )
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == source
            && site.source_callee == "this.load"
            && matches!(
                &site.target,
                ResolvedCallTarget::RepositoryFunction { scope, .. } if scope == "StaticService/load"
            )
    }));
}

#[test]
fn static_getter_this_member_resolves_as_static() {
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
    let source = root.join("src/this-member-static-getter.mts");
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
fn sibling_same_named_classes_leave_this_member_unresolved() {
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
    let source = root.join("src/this-member-sibling.mts");
    assert!(!graph.resolved_call_sites().iter().any(|site| {
        site.file == source
            && site.source_callee == "this.load"
            && matches!(&site.target, ResolvedCallTarget::RepositoryFunction { .. })
    }));
}

#[test]
fn cyclic_heritage_leaves_missing_this_member_unresolved() {
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
    let source = root.join("src/this-member-cycle.mts");
    assert!(!graph.resolved_call_sites().iter().any(|site| {
        site.file == source
            && site.source_callee == "this.missing"
            && matches!(&site.target, ResolvedCallTarget::RepositoryFunction { .. })
    }));
}

#[test]
fn mixed_kind_this_member_stays_unresolved_on_the_other_kind() {
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
    let source = root.join("src/this-member-wrong-kind.mts");
    assert!(!graph.resolved_call_sites().iter().any(|site| {
        site.file == source
            && matches!(site.source_callee.as_str(), "this.load" | "this.run")
            && matches!(&site.target, ResolvedCallTarget::RepositoryFunction { .. })
    }));
}
