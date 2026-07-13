use super::*;

pub(crate) fn analyze_selectors_with_policy(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
) -> Result<Analysis> {
    let snapshot = VisiblePathSnapshot::new(root);
    analyze_selectors_with_policy_from_snapshot(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        &snapshot,
    )
}

pub(crate) fn analyze_selectors_with_policy_and_graph(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    route_import_candidate: Option<(
        &crate::codebase::dependencies::graph::DepGraph,
        &crate::codebase::ts_resolver::TsConfig,
    )>,
    graph_file_universe: &[PathBuf],
) -> Result<Analysis> {
    let snapshot = VisiblePathSnapshot::from_paths(root, graph_file_universe);
    analyze_selectors_with_policy_and_graph_from_snapshot(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        route_import_candidate,
        graph_file_universe,
        &snapshot,
    )
}

pub(crate) fn analyze_selectors_with_policy_and_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
) -> Result<Analysis> {
    let snapshot = VisiblePathSnapshot::new(root);
    analyze_selectors_with_policy_and_facts_from_snapshot(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        facts,
        &snapshot,
    )
}

pub(crate) fn analyze_selectors_with_policy_facts_and_graph(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
    route_import_candidate: Option<(
        &crate::codebase::dependencies::graph::DepGraph,
        &crate::codebase::ts_resolver::TsConfig,
    )>,
    graph_file_universe: &[PathBuf],
) -> Result<Analysis> {
    let snapshot = VisiblePathSnapshot::from_paths(root, graph_file_universe);
    analyze_selectors_with_policy_facts_and_graph_from_snapshot(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        SelectorFactsGraphInputs {
            facts,
            route_import_candidate,
            graph_file_universe,
            snapshot: &snapshot,
        },
    )
}
