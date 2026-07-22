use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::pipeline::{
    analyze_selectors_with_policy_and_facts_from_snapshot,
    analyze_selectors_with_policy_from_snapshot, analyze_with_policy_and_facts_from_snapshot,
    analyze_with_policy_from_snapshot,
};
use crate::playwright::config;
use crate::playwright::playwright_tests;
use crate::playwright::rule_findings::findings_from_report;
use anyhow::Result;
use filter::filter_rule_findings;
pub use policy::configured;
use policy::unique_policy;
use selection::rule_selections;
use std::path::Path;

mod fact_plan;
mod filter;
mod policy;
mod prepared;
mod prepared_entrypoints;
mod selection;

pub use fact_plan::{fact_plan_for_consumers, PlaywrightFactConsumers};
pub use prepared::PreparedPlaywrightRules;
pub use prepared_entrypoints::{
    fact_plan, prepare, prepare_from_snapshot, prepare_from_snapshot_with_catalog,
};

pub const PLAYWRIGHT_COVERAGE: &str = "playwright-coverage";
pub const PLAYWRIGHT_UNIQUE_TEST_IDS: &str = "playwright-unique-test-ids";
pub const PLAYWRIGHT_UNIQUE_HTML_IDS: &str = "playwright-unique-html-ids";
pub const PLAYWRIGHT_PREFER_TEST_ID_LOCATORS: &str = "playwright-prefer-test-id-locators";

pub fn check(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    let selections = rule_selections(config);
    if selections.is_empty() {
        return Ok(Vec::new());
    }

    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    let mut findings = Vec::new();
    for selection in selections {
        let settings = config::load_settings_from_visible(
            root,
            config_path,
            &[],
            selection.playwright_project.clone(),
            &snapshot,
        )?;
        let test_policy = playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        };
        let unique_policy = unique_policy(selection.unique_test_ids, selection.unique_html_ids);
        let analysis = if selection.coverage {
            analyze_with_policy_from_snapshot(
                root,
                &settings,
                test_policy,
                unique_policy,
                &snapshot,
            )
        } else {
            analyze_selectors_with_policy_from_snapshot(
                root,
                &settings,
                test_policy,
                unique_policy,
                &snapshot,
            )
        }?;
        let report_findings = findings_from_report(
            &analysis,
            selection.coverage,
            selection.unique_test_ids,
            selection.unique_html_ids,
            selection.prefer_test_id_locators,
        );
        findings.extend(filter_rule_findings(root, config, report_findings)?);
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

pub(crate) fn check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    check_with_facts_from_snapshot(root, config_path, config, facts, &snapshot)
}

pub(crate) fn check_with_prepared_facts(
    root: &Path,
    _config_path: Option<&Path>,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    prepared: &PreparedPlaywrightRules,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for prepared_selection in &prepared.selections {
        findings.extend(check_selection_with_facts(
            root,
            config,
            facts,
            prepared.snapshot.as_ref(),
            &prepared_selection.selection,
            &prepared_selection.settings,
        )?);
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn check_with_facts_from_snapshot(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    let selections = rule_selections(config);
    if selections.is_empty() {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();
    for selection in selections {
        let settings = config::load_settings_from_visible(
            root,
            config_path,
            &[],
            selection.playwright_project.clone(),
            snapshot,
        )?;
        findings.extend(check_selection_with_facts(
            root, config, facts, snapshot, &selection, &settings,
        )?);
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn check_selection_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
    selection: &selection::RuleSelection,
    settings: &config::Settings,
) -> Result<Vec<RuleFinding>> {
    let test_policy = playwright_tests::TestPolicy {
        assert_conditional_tests: false,
        allow_skipped_tests: false,
    };
    let unique_policy = unique_policy(selection.unique_test_ids, selection.unique_html_ids);
    let analysis = if selection.coverage {
        analyze_with_policy_and_facts_from_snapshot(
            root,
            settings,
            test_policy,
            unique_policy,
            facts,
            snapshot,
        )
    } else {
        analyze_selectors_with_policy_and_facts_from_snapshot(
            root,
            settings,
            test_policy,
            unique_policy,
            facts,
            snapshot,
        )
    }?;
    let report_findings = findings_from_report(
        &analysis,
        selection.coverage,
        selection.unique_test_ids,
        selection.unique_html_ids,
        selection.prefer_test_id_locators,
    );
    filter_rule_findings(root, config, report_findings)
}

#[cfg(test)]
mod tests;
