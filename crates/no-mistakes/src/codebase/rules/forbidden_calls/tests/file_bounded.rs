#[test]
fn file_traversal_scans_only_selected_files() {
    let selection = include_str!("../selection.rs");
    assert!(
        selection.contains("graph.call_sites_in_file(&file)"),
        "file traversal must iterate selected files' call sites"
    );
    assert!(
        selection.contains(
            "graph.call_source_nodes(roots, CallTraversal::Transitive, options.max_depth)"
        ),
        "source selection must share the bounded union traversal"
    );
    assert!(
        selection.contains("return graph.call_source_nodes("),
        "transitive source selection must return before the legacy direct trace path"
    );
}

#[test]
fn file_traversal_filters_each_root_file_once_before_scanning_calls() {
    use super::*;
    let root = coverage_root();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut app = application(
        "filtered",
        "roots: [{ file: src/entry.mts }, { file: src/selectors.mts }, { function: { file: src/entry.mts, symbol: entry } }]\ntraversal: file\ntargets: [{ global: setTimeout }]",
    );
    app.include = vec!["src/entry.mts".to_string()];
    let config = NoMistakesConfig {
        rules: vec![app],
        ..Default::default()
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let findings = crate::diagnostics::with_observer(Some(observer.clone()), || {
        check_with_graph(&root, &config, &graph, None, None, &[], None).unwrap()
    });
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/entry.mts");
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("forbidden_calls.path_filter_checks"),
        Some(&2),
        "filter each unique selected file, including the excluded file"
    );
}

#[test]
fn transitive_rule_uses_one_source_union_for_overlapping_roots() {
    use super::*;
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/graph-call-source-nodes/fixture"),
    );
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    let config = NoMistakesConfig {
        rules: vec![application(
            "overlapping",
            "roots: [{ function: { file: calls.ts, symbol: alpha } }, { function: { file: calls.ts, symbol: beta } }]\ntraversal: transitive\ntargets: [{ global: setTimeout }]",
        )],
        ..Default::default()
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let findings = crate::diagnostics::with_observer(Some(observer.clone()), || {
        check_with_graph(&root, &config, &graph, None, None, &[], None).unwrap()
    });
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "calls.ts");
    let options: Options = config.rules[0].try_rule_options().unwrap();
    let roots = support::expand_roots(&root, &options, &graph, &[]).unwrap();
    let sources = graph.call_source_nodes(
        &roots,
        crate::codebase::dependencies::graph::CallTraversal::Transitive,
        None,
    );
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("graph.call_source_nodes.expanded"),
        Some(&(sources.len() as u64)),
        "the production rule must select sources once, without per-root traces"
    );
}
