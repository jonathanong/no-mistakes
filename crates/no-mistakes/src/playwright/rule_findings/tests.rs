use super::*;
use crate::playwright::analysis::types::{
    Analysis, CoverageFetch, CoverageRoute, CoverageSelector, Edge, EdgeReport, SelectorRef,
    Summary,
};
use std::sync::Arc;

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
            helper_references: vec![],
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

    let findings = findings_from_report(&analysis(report, vec![]), true, true, true, false);
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

    let findings = findings_from_report(&analysis(report, vec![]), false, false, true, false);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, PLAYWRIGHT_UNIQUE_HTML_IDS);
}

#[test]
fn findings_from_report_skips_unsupported_dynamic_selectors() {
    // Selectors with unsupported_dynamic: true (fully-dynamic expressions like
    // `data-pw={id}`) must NOT produce playwright-coverage findings.
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
                attribute: "data-pw".to_string(),
                value: "~dynamic~".to_string(),
                file: "web/app/page.tsx".to_string(),
                covered: false,
                unsupported_dynamic: true,
                tests: vec![],
                tests_detail: vec![],
                selectors: vec![],
                helper_references: vec![],
            },
            CoverageSelector {
                attribute: "data-pw".to_string(),
                value: "static-btn".to_string(),
                file: "web/app/page.tsx".to_string(),
                covered: false,
                unsupported_dynamic: false,
                tests: vec![],
                tests_detail: vec![],
                selectors: vec![],
                helper_references: vec![],
            },
        ],
        duplicate_selectors: vec![],
        fetch_apis: vec![],
    };

    let findings = findings_from_report(&analysis(report, vec![]), true, false, false, false);
    assert_eq!(
        findings.len(),
        1,
        "only the static selector should produce a finding"
    );
    assert_eq!(findings[0].target.as_deref(), Some("data-pw=static-btn"));
}

#[test]
fn findings_from_report_maps_id_duplicates_to_test_ids_when_html_rule_is_disabled() {
    let report = CoverageReport {
        summary: Summary {
            total_routes: 0,
            covered_routes: 0,
            uncovered_routes: 0,
            total_selectors: 0,
            covered_selectors: 0,
            uncovered_selectors: 0,
            duplicate_selectors: 1,
            total_fetch_apis: 0,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 0,
        },
        routes: vec![],
        selectors: vec![],
        duplicate_selectors: vec![DuplicateSelector {
            attribute: HTML_ID_ATTRIBUTE.to_string(),
            value: "save".to_string(),
            file: "web/app/page.tsx".to_string(),
        }],
        fetch_apis: vec![],
    };

    let findings = findings_from_report(&analysis(report, vec![]), false, true, false, false);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, PLAYWRIGHT_UNIQUE_TEST_IDS);
}

#[test]
fn findings_from_report_flags_copy_coupled_locators_with_selector_refs() {
    let edge = Edge::LocatorText {
        test_file: Arc::new("tests/e2e/app.spec.ts".to_string()),
        test_name: Some(Arc::new("uses text locator".to_string())),
        describe_path: Arc::new(vec![]),
        app_file: Arc::new("web/app/page.tsx".to_string()),
        locator_kind: "role".to_string(),
        role: Some("button".to_string()),
        text: "Save".to_string(),
        locator: "getByRole(button, name: Save)".to_string(),
        selector_refs: vec![SelectorRef {
            attribute: "data-pw".to_string(),
            value: "save-button".to_string(),
        }],
        reasons: vec!["route-signal".to_string()],
        line: 12,
    };

    let findings = findings_from_report(
        &analysis(empty_report(), vec![edge]),
        false,
        false,
        false,
        true,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS);
    assert_eq!(findings[0].file, "tests/e2e/app.spec.ts");
    assert_eq!(findings[0].line, 12);
    assert_eq!(findings[0].target.as_deref(), Some("data-pw=save-button"));
}

fn analysis(report: CoverageReport, edges: Vec<Edge>) -> Analysis {
    Analysis {
        coverage: report,
        edges: EdgeReport { edges },
    }
}

fn empty_report() -> CoverageReport {
    CoverageReport {
        summary: Summary {
            total_routes: 0,
            covered_routes: 0,
            uncovered_routes: 0,
            total_selectors: 0,
            covered_selectors: 0,
            uncovered_selectors: 0,
            duplicate_selectors: 0,
            total_fetch_apis: 0,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 0,
        },
        routes: vec![],
        selectors: vec![],
        duplicate_selectors: vec![],
        fetch_apis: vec![],
    }
}
