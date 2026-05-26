/// Build `EdgeKind::Selector` dep-graph edges from playwright analysis edges.
///
/// For each `PlaywrightEdge::Selector` or `PlaywrightEdge::LocatorText` edge the
/// playwright analysis pipeline emits we produce a graph edge:
///
/// ```text
/// NodeId::File(app_file)  →  NodeId::File(test_file)   EdgeKind::Selector
/// ```
///
/// This lets the test-plan coverage group select a test when the changed file
/// contains a `data-pw` selector that the test uses via `getByTestId(...)`,
/// even with no URL-navigation path.
pub(super) fn collect_playwright_selector_edges(root: &Path, all_files: &[PathBuf]) -> Vec<Edge> {
    let Ok(analysis) = run_playwright_selector_analysis(root, all_files) else {
        return vec![];
    };
    analysis
        .edges
        .edges
        .iter()
        .filter_map(|pw_edge| selector_dep_edge(root, pw_edge))
        .collect()
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
        NodeId::File(root.join(app_file_rel)),
        NodeId::File(root.join(test_file_rel)),
        EdgeKind::Selector,
    ))
}

fn run_playwright_selector_analysis(
    root: &Path,
    _all_files: &[PathBuf],
) -> anyhow::Result<crate::playwright::analysis::types::Analysis> {
    let settings = crate::playwright::config::load_settings(root, None, &[], None)?;
    let test_policy = crate::playwright::playwright_tests::TestPolicy {
        assert_conditional_tests: false,
        allow_skipped_tests: false,
    };
    let unique_policy = crate::playwright::analysis::types::UniqueSelectorPolicy::default();
    crate::playwright::analysis::pipeline::analyze_with_policy(
        root,
        &settings,
        test_policy,
        unique_policy,
    )
}
