use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::Unanalyzable;
use crate::playwright::analysis::types::Analysis;
use crate::playwright::rules::{PLAYWRIGHT_COVERAGE, PLAYWRIGHT_UNIQUE_TEST_IDS};

mod emit;

#[derive(Clone, Copy)]
pub(crate) struct CoverageFindingOptions {
    pub(crate) enabled: bool,
    pub(crate) routes: bool,
    pub(crate) selectors: bool,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LowEffortFindings {
    pub unique_test_ids: Option<Unanalyzable>,
    pub unique_html_ids: Option<Unanalyzable>,
    pub coverage: Option<Unanalyzable>,
    /// HTML IDs collected only for coverage are not configured test-ID hooks.
    pub configured_html_id_selector: bool,
}

pub(crate) struct ReportOutcome {
    pub findings: Vec<RuleFinding>,
    pub advisories: Vec<RuleFinding>,
}

pub(crate) fn findings_from_report_with_effort(
    analysis: &Analysis,
    unique_test_ids: bool,
    unique_html_ids: bool,
    prefer_test_id_locators: bool,
    coverage: CoverageFindingOptions,
    low: LowEffortFindings,
) -> ReportOutcome {
    let mut findings = Vec::new();
    let mut diagnostics = AnalysisDiagnostics::default();
    let report = &analysis.coverage;
    if coverage.enabled {
        findings.extend(emit::coverage(report, coverage, low.coverage.is_some()));
    }
    if unique_test_ids || unique_html_ids {
        findings.extend(emit::unique(
            &report.duplicate_selectors,
            unique_test_ids,
            unique_html_ids,
        ));
    }
    if unique_test_ids {
        emit::unanalyzable(
            &mut diagnostics,
            report,
            PLAYWRIGHT_UNIQUE_TEST_IDS,
            low.unique_test_ids,
            unique_html_ids || !low.configured_html_id_selector,
        );
    }
    if unique_html_ids {
        emit::unanalyzable_html_ids(
            &mut diagnostics,
            &analysis.unanalyzable_html_ids,
            low.unique_html_ids,
        );
    }
    if coverage.enabled && coverage.selectors {
        emit::unanalyzable(
            &mut diagnostics,
            report,
            PLAYWRIGHT_COVERAGE,
            low.coverage,
            false,
        );
    }
    if prefer_test_id_locators {
        findings.extend(emit::prefer_test_id_locators(analysis));
    }
    findings.extend(diagnostics.findings);
    findings.sort();
    findings.dedup();
    diagnostics.advisories.sort();
    diagnostics.advisories.dedup();
    ReportOutcome {
        findings,
        advisories: diagnostics.advisories,
    }
}

#[cfg(test)]
mod tests;
