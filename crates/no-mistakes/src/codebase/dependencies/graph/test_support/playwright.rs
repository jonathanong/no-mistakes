use super::super::*;
use std::path::{Path, PathBuf};

pub(crate) fn run_playwright_selector_analysis(
    root: &Path,
    config_path: Option<&Path>,
    facts: Option<&dyn TsFactLookup>,
    partial_graph: Option<&DepGraph>,
    graph_tsconfig: Option<&TsConfig>,
    graph_file_universe: &[PathBuf],
) -> anyhow::Result<crate::playwright::analysis::types::Analysis> {
    let snapshot =
        crate::playwright::fsutil::VisiblePathSnapshot::from_paths(root, graph_file_universe);
    let settings = crate::playwright::config::load_settings_from_visible(
        root,
        config_path,
        &[],
        None,
        None,
        &snapshot,
    );
    let settings = settings?;
    let test_policy = crate::playwright::playwright_tests::TestPolicy {
        assert_conditional_tests: false,
        allow_skipped_tests: false,
    };
    let unique_policy = crate::playwright::analysis::types::UniqueSelectorPolicy::default();
    let route_import_candidate = partial_graph.zip(graph_tsconfig);
    match facts {
        Some(facts) => crate::playwright::analysis::pipeline_selectors::analyze_selectors_with_policy_facts_and_graph_from_snapshot(
            root,
            &settings,
            test_policy,
            unique_policy,
            crate::playwright::analysis::pipeline_selectors::SelectorFactsGraphInputs {
                facts,
                route_import_candidate,
                graph_file_universe,
                snapshot: &snapshot,
            },
        ),
        None => crate::playwright::analysis::pipeline_selectors::analyze_selectors_with_policy_and_graph_from_snapshot(
            root,
            &settings,
            test_policy,
            unique_policy,
            route_import_candidate,
            graph_file_universe,
            &snapshot,
        ),
    }
}
