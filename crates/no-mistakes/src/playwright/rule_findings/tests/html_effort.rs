use super::{analysis, coverage_all, findings_from_report_with_effort, LowEffortFindings};
use crate::config::v2::Unanalyzable;
use crate::playwright::analysis::types::{CoverageReport, CoverageSelector, Summary};
use crate::playwright::rules::PLAYWRIGHT_UNIQUE_HTML_IDS;

fn selector_report() -> CoverageReport {
    let selectors: Vec<_> = [
        ("id", "literal", 4, true, false),
        ("id", "from-const", 8, false, false),
        ("id", "{id}", 12, false, true),
        ("data-testid", "{testId}", 16, false, true),
    ]
    .into_iter()
    .map(
        |(attribute, value, line, literal_exact, unsupported_dynamic)| CoverageSelector {
            attribute: attribute.to_string(),
            value: value.to_string(),
            file: "web/app/page.tsx".to_string(),
            covered: false,
            unsupported_dynamic,
            tests: vec![],
            tests_detail: vec![],
            selectors: vec![],
            helper_references: vec![],
            line,
            literal_exact,
        },
    )
    .collect();
    CoverageReport {
        warnings: Vec::new(),
        summary: Summary {
            total_routes: 0,
            covered_routes: 0,
            uncovered_routes: 0,
            total_selectors: selectors.len(),
            covered_selectors: 0,
            uncovered_selectors: selectors.len(),
            duplicate_selectors: 0,
            total_fetch_apis: 0,
            covered_fetch_apis: 0,
            uncovered_fetch_apis: 0,
        },
        routes: vec![],
        selectors,
        duplicate_selectors: vec![],
        fetch_apis: vec![],
    }
}

#[test]
fn html_low_routes_only_indirect_html_ids_through_the_requested_policy() {
    for policy in [
        Unanalyzable::Report,
        Unanalyzable::Advise,
        Unanalyzable::Ignore,
    ] {
        let outcome = findings_from_report_with_effort(
            &analysis(selector_report(), vec![]),
            false,
            true,
            false,
            coverage_all(false),
            LowEffortFindings {
                unique_html_ids: Some(policy),
                ..LowEffortFindings::default()
            },
        );
        assert_eq!(
            outcome.findings.len(),
            if policy == Unanalyzable::Report { 2 } else { 0 }
        );
        assert_eq!(
            outcome.advisories.len(),
            if policy == Unanalyzable::Advise { 2 } else { 0 }
        );
        let rows: Vec<_> = outcome.findings.iter().chain(&outcome.advisories).collect();
        if policy != Unanalyzable::Ignore {
            assert_eq!(rows.iter().map(|row| row.line).collect::<Vec<_>>(), [8, 12]);
        }
        for row in rows {
            assert_eq!(row.rule, PLAYWRIGHT_UNIQUE_HTML_IDS);
            assert!(row.message.contains("unanalyzable at effort low"));
            assert!(row
                .message
                .contains("literal HTML id such as id=\"save-button\""));
            assert!(!row.message.contains("data-pw"));
            assert!(row.target.as_deref().unwrap().starts_with("id="));
        }
    }
}

#[test]
fn html_low_diagnostics_require_the_html_rule_to_be_selected() {
    let outcome = findings_from_report_with_effort(
        &analysis(selector_report(), vec![]),
        false,
        false,
        false,
        coverage_all(false),
        LowEffortFindings {
            unique_html_ids: Some(Unanalyzable::Report),
            ..LowEffortFindings::default()
        },
    );
    assert!(outcome.findings.is_empty());
    assert!(outcome.advisories.is_empty());
}

#[test]
fn html_low_diagnostics_do_not_depend_on_coverage_selector_candidates() {
    let mut input = analysis(selector_report(), vec![]);
    // htmlIds:false keeps uniqueness occurrences while omitting coverage candidates.
    input.coverage.selectors.clear();
    input.coverage.summary.total_selectors = 0;
    input.coverage.summary.uncovered_selectors = 0;
    let serialized = serde_json::to_value(&input).unwrap();
    assert!(serialized.get("unanalyzableHtmlIds").is_none());
    assert_eq!(serialized["coverage"]["selectors"], serde_json::json!([]));

    let outcome = findings_from_report_with_effort(
        &input,
        false,
        true,
        false,
        coverage_all(false),
        LowEffortFindings {
            unique_html_ids: Some(Unanalyzable::Report),
            ..LowEffortFindings::default()
        },
    );
    assert_eq!(outcome.findings.len(), 2);
    assert!(outcome.advisories.is_empty());
    assert_eq!(
        outcome
            .findings
            .iter()
            .map(|row| row.line)
            .collect::<Vec<_>>(),
        [8, 12],
    );
}
