use super::{analysis, coverage_all, findings_from_report_with_effort, LowEffortFindings};
use crate::config::v2::Unanalyzable;
use crate::playwright::analysis::types::{CoverageReport, CoverageSelector, Summary};
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;

#[test]
fn low_effort_emits_unanalyzable_for_non_literal_exact_selectors() {
    let report = CoverageReport {
        summary: Summary {
            total_routes: 0,
            covered_routes: 0,
            uncovered_routes: 0,
            total_selectors: 2,
            covered_selectors: 0,
            uncovered_selectors: 2,
            duplicate_selectors: 0,
            total_fetch_apis: 0,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 0,
        },
        routes: vec![],
        selectors: vec![
            CoverageSelector {
                attribute: "data-testid".to_string(),
                value: "save".to_string(),
                file: "web/app/page.tsx".to_string(),
                covered: false,
                unsupported_dynamic: false,
                tests: vec![],
                tests_detail: vec![],
                selectors: vec![],
                helper_references: vec![],
                line: 4,
                literal_exact: true,
            },
            CoverageSelector {
                attribute: "data-testid".to_string(),
                value: "{ID}".to_string(),
                file: "web/app/page.tsx".to_string(),
                covered: false,
                unsupported_dynamic: true,
                tests: vec![],
                tests_detail: vec![],
                selectors: vec![],
                helper_references: vec![],
                line: 9,
                literal_exact: false,
            },
        ],
        duplicate_selectors: vec![],
        fetch_apis: vec![],
    };
    let outcome = findings_from_report_with_effort(
        &analysis(report, vec![]),
        true,
        false,
        false,
        coverage_all(false),
        LowEffortFindings {
            unique_html_ids: None,
            unique_test_ids: Some(Unanalyzable::Report),
            coverage: None,
            ..LowEffortFindings::default()
        },
    );
    assert!(outcome.advisories.is_empty());
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(outcome.findings[0].line, 9);
    assert!(outcome.findings[0]
        .message
        .contains("unanalyzable at effort low"));
}

#[test]
fn low_effort_skips_html_ids_when_the_html_rule_is_enabled() {
    let report = CoverageReport {
        summary: Summary {
            total_routes: 0,
            covered_routes: 0,
            uncovered_routes: 0,
            total_selectors: 1,
            covered_selectors: 0,
            uncovered_selectors: 1,
            duplicate_selectors: 0,
            total_fetch_apis: 0,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 0,
        },
        routes: vec![],
        selectors: vec![CoverageSelector {
            attribute: HTML_ID_ATTRIBUTE.to_string(),
            value: "{id}".to_string(),
            file: "web/app/page.tsx".to_string(),
            covered: false,
            unsupported_dynamic: true,
            tests: vec![],
            tests_detail: vec![],
            selectors: vec![],
            helper_references: vec![],
            line: 12,
            literal_exact: false,
        }],
        duplicate_selectors: vec![],
        fetch_apis: vec![],
    };
    let outcome = findings_from_report_with_effort(
        &analysis(report, vec![]),
        true,
        true,
        false,
        coverage_all(false),
        LowEffortFindings {
            unique_html_ids: None,
            unique_test_ids: Some(Unanalyzable::Report),
            coverage: None,
            ..LowEffortFindings::default()
        },
    );
    assert!(outcome.findings.is_empty());
    assert!(outcome.advisories.is_empty());
}
