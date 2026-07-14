use crate::playwright::analysis::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::config;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::playwright_tests;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub(crate) mod test_support;

pub(crate) struct SelectorFactsGraphInputs<'a> {
    pub(crate) facts: &'a dyn crate::codebase::dependencies::graph::TsFactLookup,
    pub(crate) route_import_candidate: Option<(
        &'a crate::codebase::dependencies::graph::DepGraph,
        &'a crate::codebase::ts_resolver::TsConfig,
    )>,
    pub(crate) graph_file_universe: &'a [PathBuf],
    pub(crate) snapshot: &'a VisiblePathSnapshot,
}

pub(crate) fn analyze_selectors_with_policy_from_snapshot(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    snapshot: &VisiblePathSnapshot,
) -> Result<Analysis> {
    let facts =
        super::pipeline::standalone_facts(root, settings, unique_selector_policy, snapshot)?;
    super::pipeline_selectors_core::analyze_selectors_with_options(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        super::pipeline_selectors_core::SelectorAnalysisOptions {
            facts: Some(&facts),
            route_import_candidate: None,
            graph_file_universe: None,
            skip_test_file_errors: false,
            snapshot,
        },
    )
}

pub(crate) fn analyze_selectors_with_policy_and_graph_from_snapshot(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    route_import_candidate: Option<(
        &crate::codebase::dependencies::graph::DepGraph,
        &crate::codebase::ts_resolver::TsConfig,
    )>,
    graph_file_universe: &[PathBuf],
    snapshot: &VisiblePathSnapshot,
) -> Result<Analysis> {
    super::pipeline_selectors_core::analyze_selectors_with_options(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        super::pipeline_selectors_core::SelectorAnalysisOptions {
            facts: None,
            route_import_candidate,
            graph_file_universe: Some(graph_file_universe),
            skip_test_file_errors: true,
            snapshot,
        },
    )
}

pub(crate) fn analyze_selectors_with_policy_and_facts_from_snapshot(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
    snapshot: &VisiblePathSnapshot,
) -> Result<Analysis> {
    super::pipeline_selectors_core::analyze_selectors_with_options(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        super::pipeline_selectors_core::SelectorAnalysisOptions {
            facts: Some(facts),
            route_import_candidate: None,
            graph_file_universe: None,
            skip_test_file_errors: false,
            snapshot,
        },
    )
}

pub(crate) fn analyze_selectors_with_policy_facts_and_graph_from_snapshot(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    inputs: SelectorFactsGraphInputs<'_>,
) -> Result<Analysis> {
    let SelectorFactsGraphInputs {
        facts,
        route_import_candidate,
        graph_file_universe,
        snapshot,
    } = inputs;
    super::pipeline_selectors_core::analyze_selectors_with_options(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        super::pipeline_selectors_core::SelectorAnalysisOptions {
            facts: Some(facts),
            route_import_candidate,
            graph_file_universe: Some(graph_file_universe),
            skip_test_file_errors: true,
            snapshot,
        },
    )
}
