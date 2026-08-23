use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::pipeline::{
    analyze_selectors_with_policy_and_facts_from_snapshot,
    analyze_with_policy_and_facts_from_snapshot,
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
mod standalone;

pub use fact_plan::{fact_plan_for_consumers, PlaywrightFactConsumers};
pub use prepared::PreparedPlaywrightRules;
pub use prepared_entrypoints::{
    fact_plan, prepare, prepare_from_snapshot, prepare_from_snapshot_with_catalog,
};
pub use standalone::check;

pub const PLAYWRIGHT_COVERAGE: &str = "playwright-coverage";
pub const PLAYWRIGHT_UNIQUE_TEST_IDS: &str = "playwright-unique-test-ids";
pub const PLAYWRIGHT_UNIQUE_HTML_IDS: &str = "playwright-unique-html-ids";
pub const PLAYWRIGHT_PREFER_TEST_ID_LOCATORS: &str = "playwright-prefer-test-id-locators";

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
        let next = check_selection_with_facts(
            root,
            config,
            facts,
            prepared.snapshot.as_ref(),
            &prepared_selection.selection,
            &prepared_selection.settings,
        );
        findings.extend(next?);
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
    let root_paths = snapshot.paths_for(root);
    let apps = crate::config::v2::frontend_apps(root, config, &root_paths)?;
    let selections = rule_selections(config, &apps)?;
    if selections.is_empty() {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();
    for selection in selections {
        // Deliberately reloaded from `config_path` rather than derived from
        // `config` — see the doc comment on `standalone::check`.
        let settings = config::load_settings_from_visible(
            root,
            config_path,
            &[],
            selection.playwright_project.clone(),
            selection.app.clone(),
            snapshot,
        );
        let settings = settings?;
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
        selection.unique_test_ids,
        selection.unique_html_ids,
        selection.prefer_test_id_locators,
        crate::playwright::rule_findings::CoverageFindingOptions {
            enabled: selection.coverage,
            routes: selection.cover_routes,
            selectors: selection.cover_selectors,
        },
    );
    filter_rule_findings(root, config, report_findings)
}

#[cfg(test)]
mod tests;
