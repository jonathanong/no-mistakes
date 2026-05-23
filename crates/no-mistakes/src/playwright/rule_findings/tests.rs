use super::*;
use crate::playwright::analysis::types::{CoverageFetch, CoverageRoute, CoverageSelector, Summary};

#[test]
fn findings_from_report_maps_routes_selectors_and_duplicates() {
    let report = CoverageReport {
        summary: Summary {
            total_routes: 1,
            covered_routes: 0,
            uncovered_routes: 1,
            total_selectors: 1,
            covered_selectors: 0,
            uncovered_selectors: 1,
            duplicate_selectors: 2,
            total_fetch_apis: 1,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 1,
        },
        routes: vec![CoverageRoute {
            route: "/missing".to_string(),
            file: "web/app/missing/page.tsx".to_string(),
            covered: false,
            tests: vec![],
            tests_detail: vec![],
            urls: vec![],
        }],
        selectors: vec![CoverageSelector {
            attribute: "data-testid".to_string(),
            value: "missing".to_string(),
            file: "web/app/page.tsx".to_string(),
            covered: false,
            unsupported_dynamic: false,
            tests: vec![],
            tests_detail: vec![],
            selectors: vec![],
        }],
        duplicate_selectors: vec![
            DuplicateSelector {
                attribute: "data-testid".to_string(),
                value: "save".to_string(),
                file: "web/app/page.tsx".to_string(),
            },
            DuplicateSelector {
                attribute: HTML_ID_ATTRIBUTE.to_string(),
                value: "save".to_string(),
                file: "web/app/other.tsx".to_string(),
            },
        ],
        fetch_apis: vec![CoverageFetch {
            method: "GET".to_string(),
            path: "/api/missing".to_string(),
            covered: false,
            tests: vec![],
            tests_detail: vec![],
            route_files: vec!["web/app/page.tsx".to_string()],
        }],
    };

    let findings = findings_from_report(&report, true, true, true);
    let targets: Vec<_> = findings
        .iter()
        .map(|finding| (finding.rule.as_str(), finding.target.as_deref()))
        .collect();

    assert!(targets.contains(&(PLAYWRIGHT_COVERAGE, Some("/missing"))));
    assert!(targets.contains(&(PLAYWRIGHT_COVERAGE, Some("data-testid=missing"))));
    assert!(targets.contains(&(PLAYWRIGHT_UNIQUE_TEST_IDS, Some("data-testid=save"))));
    assert!(targets.contains(&(PLAYWRIGHT_UNIQUE_HTML_IDS, Some("id=save"))));
}

#[test]
fn findings_from_report_filters_disabled_duplicate_rules() {
    let report = CoverageReport {
        summary: Summary {
            total_routes: 0,
            covered_routes: 0,
            uncovered_routes: 0,
            total_selectors: 0,
            covered_selectors: 0,
            uncovered_selectors: 0,
            duplicate_selectors: 2,
            total_fetch_apis: 0,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 0,
        },
        routes: vec![],
        selectors: vec![],
        duplicate_selectors: vec![
            DuplicateSelector {
                attribute: "data-testid".to_string(),
                value: "save".to_string(),
                file: "web/app/page.tsx".to_string(),
            },
            DuplicateSelector {
                attribute: HTML_ID_ATTRIBUTE.to_string(),
                value: "save".to_string(),
                file: "web/app/other.tsx".to_string(),
            },
        ],
        fetch_apis: vec![],
    };

    let findings = findings_from_report(&report, false, false, true);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, PLAYWRIGHT_UNIQUE_HTML_IDS);
}
