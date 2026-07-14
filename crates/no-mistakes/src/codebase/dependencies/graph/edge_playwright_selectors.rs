/// Build selector dependencies after the initial graph exists so text-locator
/// reachability can reuse its RouteImport edges instead of constructing a
/// second graph.
pub(super) struct PlaywrightSelectorEdgeInputs<'a> {
    all_files: &'a [PathBuf],
    facts: Option<&'a dyn TsFactLookup>,
    partial_graph: Option<&'a DepGraph>,
    graph_tsconfig: Option<&'a TsConfig>,
    snapshot: &'a crate::playwright::fsutil::VisiblePathSnapshot,
    prepared_settings: Option<&'a crate::playwright::config::Settings>,
}

pub(super) fn collect_playwright_selector_edges_with_graph(
    root: &Path,
    config_path: Option<&Path>,
    inputs: PlaywrightSelectorEdgeInputs<'_>,
) -> Result<Vec<Edge>> {
    let analysis = run_playwright_selector_analysis_from_snapshot(root, config_path, &inputs)?;
    Ok(selector_edges_from_analysis(
        root,
        inputs.all_files,
        &analysis,
    ))
}

fn selector_edges_from_analysis(
    root: &Path,
    all_files: &[PathBuf],
    analysis: &crate::playwright::analysis::types::Analysis,
) -> Vec<Edge> {
    let file_set: std::collections::HashSet<&Path> =
        all_files.iter().map(PathBuf::as_path).collect();
    let mut edges = Vec::new();
    for playwright_edge in &analysis.edges.edges {
        if let Some((from, to, kind)) = selector_dep_edge(root, playwright_edge) {
            if from.as_file().is_some_and(|path| file_set.contains(path))
                && to.as_file().is_some_and(|path| file_set.contains(path))
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
    Some((
        NodeId::File(root.join(test_file_rel)),
        NodeId::File(root.join(app_file_rel)),
        EdgeKind::Selector,
    ))
}

fn run_playwright_selector_analysis_from_snapshot(
    root: &Path,
    config_path: Option<&Path>,
    inputs: &PlaywrightSelectorEdgeInputs<'_>,
) -> anyhow::Result<crate::playwright::analysis::types::Analysis> {
    let loaded_settings;
    let settings = if let Some(settings) = inputs.prepared_settings {
        settings
    } else {
        loaded_settings = crate::playwright::config::load_settings_from_visible(
            root,
            config_path,
            &[],
            None,
            inputs.snapshot,
        )?;
        &loaded_settings
    };
    let test_policy = crate::playwright::playwright_tests::TestPolicy {
        assert_conditional_tests: false,
        allow_skipped_tests: false,
    };
    let unique_policy = crate::playwright::analysis::types::UniqueSelectorPolicy::default();
    let route_import_candidate = inputs.partial_graph.zip(inputs.graph_tsconfig);
    match inputs.facts {
        Some(facts) => crate::playwright::analysis::pipeline_selectors::analyze_selectors_with_policy_facts_and_graph_from_snapshot(
            root,
            settings,
            test_policy,
            unique_policy,
            crate::playwright::analysis::pipeline_selectors::SelectorFactsGraphInputs {
                facts,
                route_import_candidate,
                graph_file_universe: inputs.all_files,
                snapshot: inputs.snapshot,
            },
        ),
        None => crate::playwright::analysis::pipeline_selectors::analyze_selectors_with_policy_and_graph_from_snapshot(
            root,
            settings,
            test_policy,
            unique_policy,
            route_import_candidate,
            inputs.all_files,
            inputs.snapshot,
        ),
    }
}
