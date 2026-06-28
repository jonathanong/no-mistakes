use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::rules::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use anyhow::Result;
use std::path::Path;

pub(super) fn filter_rule_findings(
    root: &Path,
    config: &NoMistakesConfig,
    findings: Vec<RuleFinding>,
) -> Result<Vec<RuleFinding>> {
    let mut filtered = Vec::new();
    for rule_id in [
        PLAYWRIGHT_COVERAGE,
        PLAYWRIGHT_UNIQUE_TEST_IDS,
        PLAYWRIGHT_UNIQUE_HTML_IDS,
        PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
    ] {
        let rule_findings = findings
            .iter()
            .filter(|finding| finding.rule == rule_id)
            .cloned()
            .collect::<Vec<_>>();
        let filtered_rule_findings = crate::codebase::rules::path_filter::filter_findings(
            root,
            config,
            rule_id,
            rule_findings,
        )?;
        filtered.extend(filtered_rule_findings);
    }
    filtered.sort();
    filtered.dedup();
    Ok(filtered)
}
