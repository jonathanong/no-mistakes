use crate::codebase::rules::RuleFinding;
use crate::playwright::analysis::types::{CoverageReport, DuplicateSelector};
use crate::playwright::rules::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_UNIQUE_HTML_IDS, PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;

pub(crate) fn findings_from_report(
    report: &CoverageReport,
    coverage: bool,
    unique_test_ids: bool,
    unique_html_ids: bool,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if coverage {
        findings.extend(coverage_findings(report));
    }
    if unique_test_ids || unique_html_ids {
        findings.extend(unique_findings(
            &report.duplicate_selectors,
            unique_test_ids,
            unique_html_ids,
        ));
    }
    findings.sort();
    findings.dedup();
    findings
}

fn coverage_findings(report: &CoverageReport) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for route in report.routes.iter().filter(|route| !route.covered) {
        findings.push(RuleFinding {
            rule: PLAYWRIGHT_COVERAGE.to_string(),
            file: route.file.clone(),
            line: 1,
            message: format!(
                "Next.js route `{}` is not covered by a Playwright navigation assertion",
                route.route
            ),
            import: None,
            target: Some(route.route.clone()),
        });
    }
    for selector in report.selectors.iter().filter(|selector| !selector.covered) {
        findings.push(RuleFinding {
            rule: PLAYWRIGHT_COVERAGE.to_string(),
            file: selector.file.clone(),
            line: 1,
            message: format!(
                "selector [{}=\"{}\"] is not covered by a Playwright locator",
                selector.attribute, selector.value
            ),
            import: None,
            target: Some(format!("{}={}", selector.attribute, selector.value)),
        });
    }
    findings
}

fn unique_findings(
    duplicates: &[DuplicateSelector],
    unique_test_ids: bool,
    unique_html_ids: bool,
) -> Vec<RuleFinding> {
    duplicates
        .iter()
        .filter_map(|selector| {
            let rule = if selector.attribute == HTML_ID_ATTRIBUTE {
                unique_html_ids
                    .then_some(PLAYWRIGHT_UNIQUE_HTML_IDS)
                    .or_else(|| unique_test_ids.then_some(PLAYWRIGHT_UNIQUE_TEST_IDS))
            } else {
                unique_test_ids.then_some(PLAYWRIGHT_UNIQUE_TEST_IDS)
            }?;
            Some(RuleFinding {
                rule: rule.to_string(),
                file: selector.file.clone(),
                line: 1,
                message: format!(
                    "selector [{}=\"{}\"] is duplicated; Playwright selectors must be unique",
                    selector.attribute, selector.value
                ),
                import: None,
                target: Some(format!("{}={}", selector.attribute, selector.value)),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
