use crate::playwright::analysis::pipeline::analyze_with_policy_and_optional_facts;
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
        false,
        None,
        None,
    )
}

pub(crate) fn analyze_selectors_with_policy_and_graph(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    route_import_graph: Option<&crate::codebase::dependencies::graph::DepGraph>,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        false,
        None,
        route_import_graph,
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
        false,
        Some(facts),
        None,
    )
}

pub(crate) fn analyze_selectors_with_policy_facts_and_graph(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
    route_import_graph: Option<&crate::codebase::dependencies::graph::DepGraph>,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        false,
        Some(facts),
        route_import_graph,
    )
}
