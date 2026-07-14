use super::analyze_with_policy_and_optional_facts as analyze_with_policy_and_optional_facts_core;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::playwright::analysis::pipeline_options::AnalysisOptions;
use crate::playwright::analysis::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::config::Settings;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::playwright_tests::TestPolicy;
use anyhow::Result;
use std::path::Path;

pub(crate) fn analyze_with_policy(
    root: &Path,
    settings: &Settings,
    test_policy: TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
) -> Result<Analysis> {
    let snapshot = VisiblePathSnapshot::new(root);
    super::analyze_with_policy_from_snapshot(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        &snapshot,
    )
}

pub(crate) fn analyze_with_policy_and_optional_facts(
    root: &Path,
    settings: &Settings,
    test_policy: TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    require_routes: bool,
    facts: Option<&dyn TsFactLookup>,
) -> Result<Analysis> {
    let snapshot = VisiblePathSnapshot::new(root);
    analyze_with_policy_and_optional_facts_core(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        AnalysisOptions {
            require_routes,
            skip_test_file_errors: false,
            facts,
            route_import_candidate: None,
            graph_file_universe: None,
            occurrence_selection:
                super::super::pipeline_occurrences::CachedOccurrenceSelection::Exact,
            snapshot: &snapshot,
        },
    )
}
