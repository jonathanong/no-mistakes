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
    )
}

pub(crate) fn analyze_selectors_with_policy_and_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        false,
        Some(facts),
    )
}
