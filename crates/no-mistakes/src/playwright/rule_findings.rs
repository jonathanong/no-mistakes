use crate::codebase::rules::RuleFinding;
use crate::playwright::analysis::types::{Analysis, CoverageReport, DuplicateSelector, Edge};
use crate::playwright::rules::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;
use std::collections::BTreeMap;

pub(crate) fn findings_from_report(
    analysis: &Analysis,
    coverage: bool,
    unique_test_ids: bool,
    unique_html_ids: bool,
    prefer_test_id_locators: bool,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let report = &analysis.coverage;
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
    if prefer_test_id_locators {
        findings.extend(prefer_test_id_locator_findings(analysis));
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
    for selector in report
        .selectors
        .iter()
        .filter(|selector| !selector.covered && !selector.unsupported_dynamic)
    {
        findings.push(RuleFinding {
            rule: PLAYWRIGHT_COVERAGE.to_string(),
            file: selector.file.clone(),
            line: 1,
            message: uncovered_selector_message(selector),
            import: None,
            target: Some(format!("{}={}", selector.attribute, selector.value)),
        });
    }
    findings
}

fn uncovered_selector_message(
    selector: &crate::playwright::analysis::types::CoverageSelector,
) -> String {
    let mut message = format!(
        "selector [{}=\"{}\"] is not covered by a Playwright locator",
        selector.attribute, selector.value
    );
    if let Some(reference) = selector.helper_references.first() {
        message.push_str(&format!(
            "; found '{}' in {}:{} {}, but selector coverage only counts literal getByTestId('{}') calls. Inline the getByTestId call or add explicit wrapper support.",
            selector.value, reference.test_file, reference.line, reference.call, selector.value
        ));
    }
    message
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

fn prefer_test_id_locator_findings(analysis: &Analysis) -> Vec<RuleFinding> {
    let mut by_locator = BTreeMap::new();
    for edge in &analysis.edges.edges {
        let Edge::LocatorText {
            test_file,
            locator_kind,
            locator,
            test_id_attributes,
            selector_refs,
            line,
            ..
        } = edge
        else {
            continue;
        };
        let Some(selector) = selector_refs
            .iter()
            .find(|selector| test_id_attributes.contains(&selector.attribute))
        else {
            continue;
        };
        by_locator
            .entry((test_file.as_ref().clone(), *line as usize, locator.clone()))
            .or_insert_with(|| {
                RuleFinding {
                    rule: PLAYWRIGHT_PREFER_TEST_ID_LOCATORS.to_string(),
                    file: test_file.as_ref().clone(),
                    line: *line as usize,
                    message: format!(
                        "Prefer getByTestId('{}') over copy-coupled {} locator {}; matched app element exposes {}=\"{}\"",
                        selector.value, locator_kind, locator, selector.attribute, selector.value
                    ),
                    import: None,
                    target: Some(format!("{}={}", selector.attribute, selector.value)),
                }
            });
    }
    by_locator.into_values().collect()
}

#[cfg(test)]
mod tests;
