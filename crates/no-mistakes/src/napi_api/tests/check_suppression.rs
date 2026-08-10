use super::{assert_suppression, baseline_and_audit, check_json_impl, static_check_fixture};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn check_json_skips_file_disabled_parse_errors_without_losing_other_dynamic_imports() {
    let (baseline, audit) =
        baseline_and_audit("aggregate-test-no-unmocked-dynamic-imports-disabled-parse-error");
    assert!(baseline["warnings"].as_array().is_some_and(Vec::is_empty));
    assert!(baseline["rules"].as_array().is_some_and(|findings| {
        findings.iter().any(|finding| {
            finding["rule"] == "test-no-unmocked-dynamic-imports"
                && finding["file"] == "tests/direct.test.mts"
        })
    }));
    assert!(audit["suppressed"].as_array().is_some_and(|findings| {
        findings.iter().any(|finding| {
            finding["rule"] == "test-no-unmocked-dynamic-imports"
                && finding["file"] == "tests/disabled-mock.test.mts"
        })
    }));
}

#[test]
fn check_json_reads_explicit_gitignored_test_configs_through_request_sources() {
    let root = static_check_fixture("aggregate-dynamic-import-gitignored-config");
    let output = check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value["rules"].as_array().is_some_and(|findings| {
        findings.iter().any(|finding| {
            finding["rule"] == "test-no-unmocked-dynamic-imports"
                && finding["file"] == "tests/visible.test.mts"
        })
    }));
}

#[test]
fn check_json_preserves_storybook_file_and_component_reports_when_auditing_suppression() {
    let (baseline, audit) = baseline_and_audit("aggregate-require-storybook-stories");
    assert!(baseline["rules"].as_array().is_some_and(Vec::is_empty));
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "require-storybook-stories",
            "file": "web/components/ComponentSuppressed.tsx",
            "line": 2,
            "directiveKind": "nextLine",
            "directiveLine": 1,
            "reason": "React component `ComponentSuppressed` is selected for Storybook coverage but no reachable story imports it or a parent component that renders it. Add a Storybook story, add an accepted colocated test when `allow_colocated_tests` is enabled, render it through a covered parent component, exclude it from `require-storybook-stories`, or add a documented no-mistakes disable comment.",
        }),
    );
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "require-storybook-stories",
            "file": "web/components/FileSuppressed.tsx",
            "line": 1,
            "directiveKind": "file",
            "directiveLine": 1,
            "reason": "Storybook component opt-out `components/FileSuppressed.tsx#FileSuppressed` does not match a selected component.",
        }),
    );
}

#[test]
fn check_json_audit_mode_includes_an_empty_suppression_array() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-runner/empty");
    let baseline = check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let baseline: serde_json::Value = serde_json::from_str(&baseline).unwrap();
    assert!(baseline.get("suppressed").is_none());
    let audit =
        check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap();
    let audit: serde_json::Value = serde_json::from_str(&audit).unwrap();
    assert_eq!(audit["suppressed"], json!([]));
}

#[test]
fn check_json_keeps_unsuppressed_duplicate_when_suppressed_export_sorts_first() {
    let root = static_check_fixture("suppression-unique-canonical");
    let baseline: serde_json::Value =
        serde_json::from_str(&check_json_impl(json!({ "root": root }).to_string()).unwrap())
            .unwrap();
    assert!(baseline["codebase"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["rule"] == "unique-exports" && item["file"] == "src/c.ts")
    }));
    let audit: serde_json::Value = serde_json::from_str(
        &check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap(),
    )
    .unwrap();
    assert!(audit["codebase"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["rule"] == "unique-exports" && item["file"] == "src/c.ts")
    }));
    assert!(audit["suppressed"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["rule"] == "unique-exports"
                && item["file"] == "src/a.ts"
                && item["directive"]["kind"] == "line"
                && item["directive"]["line"] == 1
        })
    }));
}

#[test]
fn check_json_propagates_origin_suppression_through_named_and_wildcard_reexports() {
    let root = static_check_fixture("suppression-unique-canonical");
    let baseline: serde_json::Value =
        serde_json::from_str(&check_json_impl(json!({ "root": root }).to_string()).unwrap())
            .unwrap();
    assert!(baseline["codebase"].as_array().is_some_and(|items| {
        !items.iter().any(|item| {
            matches!(
                item["exportName"].as_str(),
                Some("chained" | "wildOnly" | "TypeThing")
            )
        })
    }));
    let audit: serde_json::Value = serde_json::from_str(
        &check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap(),
    )
    .unwrap();
    assert!(audit["suppressed"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["rule"] == "unique-exports"
                && item["file"] == "src/named-barrel.ts"
                && item["line"] == 2
                && item["directive"]["kind"] == "nextLine"
                && item["directive"]["line"] == 1
                && item["reason"]
                    .as_str()
                    .is_some_and(|reason| reason.contains("chained"))
        }) && items.iter().any(|item| {
            item["rule"] == "unique-exports"
                && item["file"] == "shared/suppressed-origin.ts"
                && item["line"] == 5
                && item["directive"]["kind"] == "file"
                && item["directive"]["line"] == 3
                && item["reason"]
                    .as_str()
                    .is_some_and(|reason| reason.contains("wildOnly"))
        }) && items.iter().any(|item| {
            item["rule"] == "unique-exports"
                && item["file"] == "src/type-barrel.ts"
                && item["line"] == 2
                && item["directive"]["kind"] == "nextLine"
                && item["directive"]["line"] == 1
                && item["reason"]
                    .as_str()
                    .is_some_and(|reason| reason.contains("TypeThing"))
        })
    }));
}
