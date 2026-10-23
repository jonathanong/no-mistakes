use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::Unanalyzable;
use crate::playwright::analysis::types::Analysis;
use crate::playwright::rules::PLAYWRIGHT_UNIQUE_TEST_IDS;
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;

mod emit;

#[derive(Clone, Copy)]
pub(crate) struct CoverageFindingOptions {
    pub(crate) enabled: bool,
    pub(crate) routes: bool,
    pub(crate) selectors: bool,
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
    unique_test_ids_low: Option<Unanalyzable>,
) -> ReportOutcome {
    let mut findings = Vec::new();
    let mut diagnostics = AnalysisDiagnostics::default();
    let report = &analysis.coverage;
    if coverage.enabled {
        findings.extend(emit::coverage(report, coverage));
    }
    if unique_test_ids || unique_html_ids {
        findings.extend(emit::unique(
            &report.duplicate_selectors,
            unique_test_ids,
            unique_html_ids,
        ));
    }
    if let Some(policy) = unique_test_ids_low {
        for selector in &report.selectors {
            if selector.literal_exact {
                continue;
            }
            if selector.attribute == HTML_ID_ATTRIBUTE && unique_html_ids {
                continue;
            }
            diagnostics.unanalyzable(
                policy,
                RuleFinding {
                    rule: PLAYWRIGHT_UNIQUE_TEST_IDS.to_string(),
                    file: selector.file.clone(),
                    line: selector.line,
                    message: format!(
                        "selector [{}=\"{}\"] is unanalyzable at effort low; {}",
                        selector.attribute,
                        selector.value,
                        crate::playwright::rules::effort::REWRITE
                    ),
                    import: None,
                    target: Some(format!("{}={}", selector.attribute, selector.value)),
                },
            );
        }
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
