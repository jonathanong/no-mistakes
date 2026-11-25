use super::*;
use crate::napi_api::options::test_json_arg;
use serde_json::Value;

#[test]
fn check_finds_retries_in_selected_factories_and_local_playwright_fixtures() {
    let base = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/test-no-retries"),
    );
    let root = base.join("fixture/local-playwright");
    let config = base.join("configs/local-playwright.yml");
    let report: Value = serde_json::from_str(
        &check_json_impl(test_json_arg(
            json!({ "root": root, "config": config }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let batch: Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(test_json_arg(
            json!({ "root": root, "reports": [{ "type": "check", "config": config }] }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch["reports"][0]["result"], report);
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 7, "{report}");
    for (rule, file, line) in [
        ("test-no-retries", "config/browser-options.ts", 6),
        ("test-no-retries", "config/browser-options.ts", 11),
        ("test-no-retries", "tests/local.spec.ts", 7),
        ("test-no-retries", "tests/local.spec.ts", 8),
        ("playwright-no-serial", "tests/local.spec.ts", 7),
        ("playwright-no-serial", "tests/local.spec.ts", 8),
        ("playwright-no-serial", "tests/local.spec.ts", 19),
    ] {
        assert!(
            findings.iter().any(|finding| {
                finding["rule"] == rule && finding["file"] == file && finding["line"] == line
            }),
            "missing {rule} at {file}:{line}: {report}"
        );
    }
}

#[test]
fn batched_check_preserves_test_reliability_and_playwright_selector_findings() {
    let base = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/test-no-retries"),
    );
    let root = base.join("fixture/report");
    let mixed = base.join("configs/mixed.yml");
    let selectors_only = base.join("configs/selectors-only.yml");

    let check = |config| -> Value {
        serde_json::from_str(
            &check_json_impl(test_json_arg(
                json!({ "root": root, "config": config }).to_string(),
            ))
            .unwrap(),
        )
        .unwrap()
    };
    let mixed_check = check(&mixed);
    let selectors_check = check(&selectors_only);
    let mixed_findings = mixed_check["rules"].as_array().unwrap();
    for (rule, file, target) in [
        (
            "test-no-retries",
            "config/unit.config.ts",
            "Vitest test.retry",
        ),
        (
            "test-no-retries",
            "playwright.config.ts",
            "Playwright retries",
        ),
        (
            "playwright-no-serial",
            "playwright.config.ts",
            "Playwright workers: 1",
        ),
    ] {
        assert!(
            mixed_findings.iter().any(|finding| {
                finding["rule"] == rule && finding["file"] == file && finding["target"] == target
            }),
            "missing {rule} in {file} for {target}: {mixed_check}"
        );
    }
    let selectors = selectors_check["rules"].as_array().unwrap();
    assert_eq!(selectors.len(), 2, "{selectors_check}");
    assert!(selectors
        .iter()
        .all(|finding| finding["rule"] == "playwright-unique-test-ids"));
    assert_eq!(
        mixed_findings
            .iter()
            .filter(|finding| finding["rule"] == "playwright-unique-test-ids")
            .collect::<Vec<_>>(),
        selectors.iter().collect::<Vec<_>>()
    );

    let playwright: Value = serde_json::from_str(
        &playwright_check_json_impl(test_json_arg(
            json!({ "root": root, "config": mixed, "assertUniqueTestIds": true }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let batch: Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(test_json_arg(
            json!({ "root": root, "reports": [
                { "type": "check", "config": mixed },
                { "type": "check", "config": selectors_only },
                { "type": "playwrightCheck", "config": mixed, "assertUniqueTestIds": true },
            ] })
            .to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch["reports"].as_array().unwrap().len(), 3);
    assert_eq!(batch["reports"][0]["result"], mixed_check);
    assert_eq!(batch["reports"][1]["result"], selectors_check);
    assert_eq!(batch["reports"][2]["result"], playwright);
}

#[test]
fn check_follows_delegated_wrappers_and_reports_unprovable_bindings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-reliability/wrapped-coverage"),
    );
    let report: Value = serde_json::from_str(
        &check_json_impl(test_json_arg(json!({ "root": root }).to_string())).unwrap(),
    )
    .unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{report}");
    let incomplete = report["incomplete"].as_array().unwrap();
    assert_eq!(incomplete.len(), 12, "{report}");
}
