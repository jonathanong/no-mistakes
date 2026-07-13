use crate::playwright::analysis::pipeline::analyze_with_policy_and_optional_facts;
use crate::playwright::analysis::pipeline_options::AnalysisOptions;
use crate::playwright::analysis::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::config;
use crate::playwright::playwright_tests;
use anyhow::Result;
use std::path::Path;

pub(crate) fn analyze_selectors_with_policy(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes: false,
            skip_test_file_errors: false,
            facts: None,
            route_import_candidate: None,
            graph_file_universe: None,
            occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
        },
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
    graph_file_universe: &[std::path::PathBuf],
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes: false,
            skip_test_file_errors: true,
            facts: None,
            route_import_candidate,
            graph_file_universe: Some(graph_file_universe),
            occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
        },
    )
}

pub(crate) fn analyze_selectors_with_policy_and_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes: false,
            skip_test_file_errors: false,
            facts: Some(facts),
            route_import_candidate: None,
            graph_file_universe: None,
            occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
        },
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
    graph_file_universe: &[std::path::PathBuf],
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes: false,
            skip_test_file_errors: true,
            facts: Some(facts),
            route_import_candidate,
            graph_file_universe: Some(graph_file_universe),
            occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
        },
    )
}
