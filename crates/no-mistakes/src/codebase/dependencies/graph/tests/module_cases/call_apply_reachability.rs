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

fn keeps_loaded(file: &str) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let deps = dynamic_import_deps(file);
    assert!(
        deps.contains(root.join("src/call-apply-loaded.mts").as_path()),
        "{file} should keep Function.prototype call/apply imports"
    );
}

#[test]
fn local_call_keeps_function_scoped_imports() {
    keeps_loaded("src/call-apply-local.mts");
}

#[test]
fn local_apply_keeps_function_scoped_imports() {
    keeps_loaded("src/call-apply-apply.mts");
}

#[test]
fn aliased_call_keeps_function_scoped_imports() {
    keeps_loaded("src/call-apply-aliased.mts");
}

#[test]
fn imported_call_resolves_to_the_exported_callable() {
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
    let consumer = root.join("src/call-apply-imported.mts");
    let target = root.join("src/call-apply-export.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: consumer.clone(),
        symbol: "run".to_string(),
    }]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(
        traces.iter().any(|trace| matches!(
            &trace.target,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "target"
        )),
        "imported Function.prototype.call must resolve to the exported callable"
    );
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == consumer
            && site.source_callee == "target"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    repository_target: Some((file, scope)),
                    ..
                } if file == &target && scope == "target"
            )
    }));
}

#[test]
fn dynamic_receiver_call_stays_conservative() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let deps = dynamic_import_deps("src/call-apply-dynamic.mts");
    assert!(!deps.contains(root.join("src/call-apply-loaded.mts").as_path()));
}
