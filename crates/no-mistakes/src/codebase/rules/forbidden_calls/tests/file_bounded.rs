#[test]
fn file_traversal_scans_only_selected_files() {
    let selection = include_str!("../selection.rs");
    assert!(
        selection.contains("graph.call_sites_in_file(&file)"),
        "file traversal must iterate selected files' call sites"
    );
    assert!(
        selection.contains("file traversal returns the expanded roots"),
        "file traversal must not BFS the unselected universe"
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
fn file_traversal_keeps_function_roots_scoped_within_a_selected_file() {
    use super::*;
    let findings = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/selectors.mts, symbol: timeoutCall } }]\n      traversal: file\n      targets: [{ terminal: waitForTimeout }, { terminal: setTimeout }]\n",
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].import.as_deref(), Some("page.waitForTimeout"));
}
