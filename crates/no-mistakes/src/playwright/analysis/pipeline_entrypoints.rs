use super::pipeline::{analyze_with_policy_and_optional_facts, standalone_facts};
use super::pipeline_options::AnalysisOptions;
use super::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::{config, playwright_tests};
use anyhow::Result;
use std::path::Path;

pub(crate) fn analyze_with_policy_from_snapshot(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    snapshot: &VisiblePathSnapshot,
) -> Result<Analysis> {
    let facts = standalone_facts(root, settings, unique_selector_policy, snapshot)?;
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes: true,
            skip_test_file_errors: false,
            facts: Some(&facts),
            route_import_candidate: None,
            graph_file_universe: None,
            occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
            snapshot,
        },
    )
}

pub(crate) fn analyze_with_policy_and_facts_from_snapshot(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
    snapshot: &VisiblePathSnapshot,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes: true,
            skip_test_file_errors: false,
            facts: Some(facts),
            route_import_candidate: None,
            graph_file_universe: None,
            occurrence_selection: super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
            snapshot,
        },
    )
}
