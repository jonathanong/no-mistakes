use super::{
    analysis, coverage_all, findings_from_report_with_effort, CoverageFindingOptions,
    LowEffortFindings,
};
use crate::config::v2::Unanalyzable;
use crate::playwright::analysis::types::{CoverageReport, CoverageSelector, Summary};
use crate::playwright::rules::PLAYWRIGHT_COVERAGE;

#[test]
fn low_coverage_skips_recovered_uncovered_selectors() {
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
                value: "from-const".to_string(),
                file: "web/app/page.tsx".to_string(),
                covered: false,
                unsupported_dynamic: false,
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
        false,
        false,
        false,
        coverage_all(true),
        LowEffortFindings {
            unique_test_ids: None,
            coverage: Some(Unanalyzable::Report),
        },
    );
    assert!(outcome.advisories.is_empty());
    let coverage: Vec<_> = outcome
        .findings
        .iter()
        .filter(|row| row.message.contains("is not covered"))
        .collect();
    assert_eq!(coverage.len(), 1);
    assert_eq!(coverage[0].target.as_deref(), Some("data-testid=save"));
    assert!(outcome.findings.iter().any(|row| {
        row.rule == PLAYWRIGHT_COVERAGE
            && row.line == 9
            && row.message.contains("unanalyzable at effort low")
    }));
}

#[test]
fn low_coverage_skips_unanalyzable_when_selector_coverage_is_disabled() {
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
            attribute: "data-testid".to_string(),
            value: "{id}".to_string(),
            file: "web/app/page.tsx".to_string(),
            covered: false,
            unsupported_dynamic: true,
            tests: vec![],
            tests_detail: vec![],
            selectors: vec![],
            helper_references: vec![],
            line: 9,
            literal_exact: false,
        }],
        duplicate_selectors: vec![],
        fetch_apis: vec![],
    };
    let outcome = findings_from_report_with_effort(
        &analysis(report, vec![]),
        false,
        false,
        false,
        CoverageFindingOptions {
            enabled: true,
            routes: true,
            selectors: false,
        },
        LowEffortFindings {
            unique_test_ids: None,
            coverage: Some(Unanalyzable::Report),
        },
    );
    assert!(outcome.findings.is_empty());
    assert!(outcome.advisories.is_empty());
}
