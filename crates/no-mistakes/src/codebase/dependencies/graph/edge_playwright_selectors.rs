/// Build `EdgeKind::Selector` dep-graph edges from playwright analysis edges.
///
/// For each `PlaywrightEdge::Selector` or `PlaywrightEdge::LocatorText` edge the
/// playwright analysis pipeline emits we produce a graph edge:
///
/// ```text
/// NodeId::File(test_file)  →  NodeId::File(app_file)   EdgeKind::Selector
/// ```
///
/// The direction mirrors `EdgeKind::TestOf` (test depends on source) so that
/// `dependents_of(app_file)` returns tests that cover it via selector-based
/// paths, even with no URL-navigation route connecting them.
pub(super) fn collect_playwright_selector_edges_with_graph(
    root: &Path,
    config_path: Option<&Path>,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
    partial_graph: Option<&DepGraph>,
    graph_tsconfig: Option<&TsConfig>,
) -> Result<Vec<Edge>> {
    let analysis = run_playwright_selector_analysis(
        root,
        config_path,
        facts,
        partial_graph,
        graph_tsconfig,
        all_files,
    )?;
    Ok(selector_edges_from_analysis(root, all_files, &analysis))
}

fn selector_edges_from_analysis(
    root: &Path,
    all_files: &[PathBuf],
    analysis: &crate::playwright::analysis::types::Analysis,
) -> Vec<Edge> {
    // Use the graph's pre-discovered file set to filter: only emit edges whose
    // both endpoints are files the dep-graph already knows about.  This avoids
    // introducing nodes outside the graph's file set and avoids a second
    // filesystem walk on top of the one the dep-graph builder already did.
    let file_set: std::collections::HashSet<&Path> =
        all_files.iter().map(PathBuf::as_path).collect();
    let mut edges = Vec::new();
    for pw_edge in &analysis.edges.edges {
        if let Some((from, to, kind)) = selector_dep_edge(root, pw_edge) {
            // selector_dep_edge always produces File nodes; both must be in the
            // graph's file set so we don't introduce phantom nodes.
            if from.as_file().is_some_and(|p| file_set.contains(p))
                && to.as_file().is_some_and(|p| file_set.contains(p))
            {
                edges.push((from, to, kind));
            }
        }
    }
    edges
}

fn selector_dep_edge(root: &Path, edge: &crate::playwright::analysis::types::Edge) -> Option<Edge> {
    let (app_file_rel, test_file_rel) = match edge {
        crate::playwright::analysis::types::Edge::Selector {
            app_file,
            test_file,
            ..
        } => (app_file.as_str(), test_file.as_str()),
        crate::playwright::analysis::types::Edge::LocatorText {
            app_file,
            test_file,
            ..
        } => (app_file.as_str(), test_file.as_str()),
        _ => return None,
    };
    // Edge direction: test_file → app_file, mirroring how TestOf edges work
    // (test depends on source).  The reverse map then gives "dependents of
    // app_file" → test files that cover it via selector-based paths.
    Some((
        NodeId::File(root.join(test_file_rel)),
        NodeId::File(root.join(app_file_rel)),
        EdgeKind::Selector,
    ))
}

fn run_playwright_selector_analysis(
    root: &Path,
    config_path: Option<&Path>,
    facts: Option<&dyn TsFactLookup>,
    partial_graph: Option<&DepGraph>,
    graph_tsconfig: Option<&TsConfig>,
    graph_file_universe: &[PathBuf],
) -> anyhow::Result<crate::playwright::analysis::types::Analysis> {
    // Reuse the same config file the rest of this DepGraph build (and, when
    // called from `check`, the sibling `playwright` rule) resolved settings
    // from — previously hardcoded to `None`, which silently ignored an
    // explicit `--config` and fell back to default-discovery instead. That
    // divergence would also have made sharing the app-wide selector scan
    // between this path and the `playwright` rule's path unsafe.
    let settings = crate::playwright::config::load_settings(root, config_path, &[], None)?;
    let test_policy = crate::playwright::playwright_tests::TestPolicy {
        assert_conditional_tests: false,
        allow_skipped_tests: false,
    };
    let unique_policy = crate::playwright::analysis::types::UniqueSelectorPolicy::default();
    let route_import_candidate = partial_graph.zip(graph_tsconfig);
    // Use the selectors-only pipeline: does not require Next.js routes to exist,
    // so selector edges work for components that have no direct route coverage.
    // Reuse already-collected Playwright test-file facts when the caller has
    // them (e.g. `check`'s shared CheckFactMap) instead of re-parsing and
    // re-analyzing every test file from scratch.
    match facts {
        Some(facts) => crate::playwright::analysis::pipeline_selectors::analyze_selectors_with_policy_facts_and_graph(
            root,
            &settings,
            test_policy,
            unique_policy,
            facts,
            route_import_candidate,
            graph_file_universe,
        ),
        None => crate::playwright::analysis::pipeline_selectors::analyze_selectors_with_policy_and_graph(
            root,
            &settings,
            test_policy,
            unique_policy,
            route_import_candidate,
            graph_file_universe,
        ),
    }
}
