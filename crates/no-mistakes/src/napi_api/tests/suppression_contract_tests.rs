use std::path::PathBuf;

use serde_json::json;

use super::super::check_json_impl;

#[test]
fn check_json_reports_suppressed_findings_from_prepared_root_fixtures() {
    let fixtures = [
        ("suppression-react", "react", "assert-no-fetch"),
        ("suppression-unique-canonical", "codebase", "unique-exports"),
        (
            "aggregate-agents-md-advisory-suppression",
            "advisories",
            "agents-md-max-size",
        ),
        (
            "aggregate-test-no-unmocked-dynamic-imports",
            "rules",
            "test-no-unmocked-dynamic-imports",
        ),
    ];

    for (fixture, domain, rule) in fixtures {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check")
            .join(fixture);
        let output =
            check_json_impl(crate::napi_api::options::test_json_arg(json!({ "root": root, "includeSuppressed": true }).to_string()))
                .unwrap();
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(
            report["suppressed"].as_array().is_some_and(|findings| {
                findings
                    .iter()
                    .any(|finding| finding["domain"] == domain && finding["rule"] == rule)
            }),
            "fixture {fixture}: {report}"
        );
    }
}
