use super::*;
use std::collections::BTreeSet;

fn workspace_call_graph() -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("call-workspace-packages"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    // Calls alone: the workspace map must be loaded for the call plan itself,
    // not borrowed from an import, workspace, or symbol plan.
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn call_site<'a>(graph: &'a DepGraph, file: &Path, callee: &str) -> &'a ResolvedCallSite {
    graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.file == file && site.source_callee == callee)
        .unwrap_or_else(|| panic!("no call site `{callee}` in {}", file.display()))
}

/// Direct `Call` edge targets of every callable declared in `file`.
fn direct_call_targets(graph: &DepGraph, file: &Path) -> BTreeSet<(PathBuf, String)> {
    let roots = graph.expand_call_roots(&[CallRoot::File(file.to_path_buf())]);
    graph
        .call_traces(&roots, CallTraversal::Direct, None)
        .into_iter()
        .filter_map(|trace| match trace.target {
            NodeId::Symbol { file, symbol, .. } => Some((file.to_path_buf(), symbol.to_string())),
            _ => None,
        })
        .collect()
}

#[test]
fn calls_through_workspace_package_names_resolve_to_repository_callables() {
    let (root, graph) = workspace_call_graph();
    let entry = root.join("src/entry.ts");
    let lib = root.join("packages/lib");

    // The package root, an `exports` subpath, a namespace import of that
    // subpath, and a subpath barrel that re-exports from a sibling file.
    for (callee, specifier, file) in [
        ("libFn", "@scope/lib", "index.ts"),
        ("utilFn", "@scope/lib/util", "src/util.ts"),
        ("util.utilFn", "@scope/lib/util", "src/util.ts"),
        ("deepFn", "@scope/lib/barrel", "src/deep.ts"),
    ] {
        let site = call_site(&graph, &entry, callee);
        let export = callee.rsplit('.').next().unwrap();
        assert!(
            matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier: found_specifier,
                    export_path,
                    repository_target: Some((found_file, scope)),
                    ..
                } if found_specifier == specifier
                    && export_path == export
                    && found_file == &lib.join(file)
                    && scope == export
            ),
            "{callee} must resolve to its workspace package source: {site:#?}"
        );
    }

    let expected = [
        ("index.ts", "libFn"),
        ("src/util.ts", "utilFn"),
        ("src/deep.ts", "deepFn"),
    ]
    .into_iter()
    .map(|(file, symbol)| (lib.join(file), symbol.to_string()))
    .collect::<BTreeSet<_>>();
    assert_eq!(
        direct_call_targets(&graph, &entry),
        expected,
        "only the three resolvable workspace calls become Call edges"
    );
}

#[test]
fn bare_external_and_unknown_package_specifiers_stay_unresolved() {
    let (root, graph) = workspace_call_graph();
    let entry = root.join("src/entry.ts");

    for (callee, specifier) in [("useState", "react"), ("missing", "@scope/missing")] {
        let site = call_site(&graph, &entry, callee);
        assert!(
            matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier: found_specifier,
                    export_path,
                    repository_target: None,
                    callable_id: None,
                } if found_specifier == specifier && export_path == callee
            ),
            "{callee} from {specifier} must stay an unresolved module export: {site:#?}"
        );
    }
}

#[test]
fn non_callable_workspace_export_is_an_unknown_call_like_a_relative_import() {
    let (root, graph) = workspace_call_graph();
    let site = call_site(&graph, &root.join("src/entry.ts"), "answer");

    assert!(
        matches!(site.target, ResolvedCallTarget::Unknown),
        "a resolved workspace module whose export is not callable is unknown: {site:#?}"
    );
}

#[test]
fn workspace_reexports_resolve_for_relative_importers_and_call_roots() {
    let (root, graph) = workspace_call_graph();
    let consumer = root.join("src/reexport-consumer.ts");
    let index = root.join("packages/lib/index.ts");

    let site = call_site(&graph, &consumer, "reexported");
    assert!(
        matches!(
            &site.target,
            ResolvedCallTarget::ModuleExport {
                specifier,
                export_path,
                repository_target: Some((file, scope)),
                ..
            } if specifier == "./reexport"
                && export_path == "reexported"
                && file == &index
                && scope == "libFn"
        ),
        "a relative import must follow the re-export through the workspace package: {site:#?}"
    );
    assert_eq!(
        direct_call_targets(&graph, &consumer),
        BTreeSet::from([(index.clone(), "libFn".to_string())])
    );

    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: root.join("src/reexport.ts"),
        symbol: "reexported".to_string(),
    }]);
    assert_eq!(roots.len(), 1, "{roots:#?}");
    assert!(has_symbol(&roots[0], &index, "libFn"), "{roots:#?}");
}
