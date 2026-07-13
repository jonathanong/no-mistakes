use super::pipeline::analyze_with_policy_and_optional_facts;
use super::pipeline_options::AnalysisOptions;
use super::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::config;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::playwright_tests;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) struct SelectorAnalysisOptions<'a> {
    pub(super) facts: Option<&'a dyn crate::codebase::dependencies::graph::TsFactLookup>,
    pub(super) route_import_candidate: Option<(
        &'a crate::codebase::dependencies::graph::DepGraph,
        &'a crate::codebase::ts_resolver::TsConfig,
    )>,
    pub(super) graph_file_universe: Option<&'a [PathBuf]>,
    pub(super) skip_test_file_errors: bool,
    pub(super) snapshot: &'a VisiblePathSnapshot,
}

pub(super) fn analyze_selectors_with_options(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    options: SelectorAnalysisOptions<'_>,
) -> Result<Analysis> {
    let SelectorAnalysisOptions {
        facts,
        route_import_candidate,
        graph_file_universe,
        skip_test_file_errors,
        snapshot,
    } = options;
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes: false,
            skip_test_file_errors,
            facts,
            route_import_candidate,
            graph_file_universe,
            occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
            snapshot,
        },
    )
}
