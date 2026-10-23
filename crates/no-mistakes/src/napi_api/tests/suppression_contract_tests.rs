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
        let output = check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "includeSuppressed": true }).to_string(),
        ))
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

#[test]
fn check_json_follows_the_importing_packages_next_config_alias() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/nextjs-config-path-alias");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": fixture.path() }).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(
        report["rules"]
            .as_array()
            .is_some_and(|findings| findings.iter().any(|finding| {
                finding["file"] == "packages/web/next-shared.ts"
                    && finding["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("redirect destination '/gone'"))
            })),
        "{report}"
    );
}

#[test]
fn next_config_check_does_not_widen_ordinary_analyze_project_reports() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/nextjs-config-path-alias-project");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let analyze = |config: &str| {
        let output = crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(json!({
                "root": fixture.path(), "config": config,
                "reports": [
                    { "type": "check", "id": "check" },
                    { "type": "dependents", "id": "ordinary", "files": ["packages/web/next-shared.ts"] },
                    { "type": "importers", "id": "importers", "file": "packages/web/next-shared.ts" }
                ]
            }).to_string())
        ).unwrap();
        serde_json::from_str::<serde_json::Value>(&output).unwrap()
    };
    let enabled = analyze(".no-mistakes.yml");
    let disabled = analyze("disabled.yml");
    assert_eq!(
        enabled["reports"][1]["result"],
        disabled["reports"][1]["result"]
    );
    assert_eq!(
        enabled["reports"][2]["result"],
        disabled["reports"][2]["result"]
    );
    assert_eq!(
        disabled["reports"][2]["result"]["directImporters"],
        json!([])
    );
}
