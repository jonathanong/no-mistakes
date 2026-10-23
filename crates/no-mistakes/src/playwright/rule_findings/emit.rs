use super::CoverageFindingOptions;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::Unanalyzable;
use crate::playwright::analysis::types::{Analysis, CoverageReport, DuplicateSelector, Edge};
use crate::playwright::rules::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;
use std::collections::BTreeMap;

pub(super) fn coverage(
    report: &CoverageReport,
    options: CoverageFindingOptions,
    literal_selectors_only: bool,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if options.routes {
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
    }
    if !options.selectors {
        return findings;
    }
    for selector in report.selectors.iter().filter(|selector| {
        !selector.covered
            && !selector.unsupported_dynamic
            && !(literal_selectors_only && !selector.literal_exact)
    }) {
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

pub(super) fn unanalyzable(
    diagnostics: &mut AnalysisDiagnostics,
    report: &CoverageReport,
    rule: &str,
    policy: Option<Unanalyzable>,
    skip_html_ids: bool,
) {
    let Some(policy) = policy else {
        return;
    };
    for selector in &report.selectors {
        if selector.literal_exact {
            continue;
        }
        if skip_html_ids && selector.attribute == HTML_ID_ATTRIBUTE {
            continue;
        }
        diagnostics.unanalyzable(
            policy,
            RuleFinding {
                rule: rule.to_string(),
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

pub(super) fn unique(
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

pub(super) fn prefer_test_id_locators(analysis: &Analysis) -> Vec<RuleFinding> {
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
