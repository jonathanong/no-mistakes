use super::effort;
use super::filter::filter_rule_findings;
use super::policy::unique_policy;
use super::selection::rule_selections;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::pipeline::{
    analyze_selectors_with_policy_from_snapshot, analyze_with_policy_from_snapshot,
};
use crate::playwright::config;
use crate::playwright::playwright_tests;
use anyhow::Result;
use std::path::Path;

/// Standalone, fully self-contained Playwright rule check: re-derives
/// Playwright settings from `config_path` rather than from the already-loaded
/// `config` (used only for rule/app selection here), so a caller that
/// constructs `config` in-memory for rule purposes can still rely on
/// on-disk Playwright settings. Not used by the aggregate `no-mistakes check`
/// pipeline — see `check_with_facts`/`check_with_prepared_facts` for that.
/// This compatibility adapter returns findings only; use aggregate check
/// diagnostics (including the Node check API) to receive non-failing advisories.
pub fn check(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    let root_paths = snapshot.paths_for(root);
    let apps = crate::config::v2::frontend_apps(root, config, &root_paths)?;
    let selections = rule_selections(config, &apps)?;
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
            selection.app.clone(),
            &snapshot,
        );
        let settings = settings?;
        let test_policy = playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        };
        let unique_ids = effort::unique_test_ids_policy(config)?;
        let html_ids = effort::unique_html_ids_policy(config)?;
        let coverage_effort = effort::coverage_policy(config)?;
        let unique_policy = unique_policy(
            selection.unique_test_ids,
            selection.unique_html_ids,
            unique_ids.low,
            html_ids.low,
        );
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
        let report = crate::playwright::rule_findings::findings_from_report_with_effort(
            &analysis,
            selection.unique_test_ids,
            selection.unique_html_ids,
            selection.prefer_test_id_locators,
            crate::playwright::rule_findings::CoverageFindingOptions {
                enabled: selection.coverage,
                routes: selection.cover_routes,
                selectors: selection.cover_selectors,
            },
            crate::playwright::rule_findings::LowEffortFindings {
                configured_html_id_selector: config::has_configured_html_id_selector(&settings),
                unique_test_ids: unique_ids.low.then_some(unique_ids.unanalyzable),
                unique_html_ids: html_ids.low.then_some(html_ids.unanalyzable),
                coverage: coverage_effort.low.then_some(coverage_effort.unanalyzable),
            },
        );
        findings.extend(filter_rule_findings(root, config, report.findings)?);
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}
