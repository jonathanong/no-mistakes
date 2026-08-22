use super::*;

#[test]
fn playwright_json_exports_return_analyzer_reports() {
    let root = fixture("nextjs-coverage", "covered");
    let check = playwright_check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let check: serde_json::Value = serde_json::from_str(&check).unwrap();
    assert!(check["summary"]["totalRoutes"].as_u64().unwrap() > 0);

    let root = fixture("nextjs-coverage", "covered");
    let edges = playwright_edges_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let edges: serde_json::Value = serde_json::from_str(&edges).unwrap();
    assert!(!edges["edges"].as_array().unwrap().is_empty());

    let root = fixture("nextjs-coverage", "covered");
    let related = playwright_related_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "files": ["web/app/settings/page.tsx"]
        })
        .to_string(),
    ))
    .unwrap();
    let related: serde_json::Value = serde_json::from_str(&related).unwrap();
    assert!(related["tests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|test| test == "tests/e2e/settings.spec.ts"));

    let root = fixture("nextjs-coverage", "covered");
    let tests = playwright_tests_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let tests: serde_json::Value = serde_json::from_str(&tests).unwrap();
    assert!(!tests["tests"].as_array().unwrap().is_empty());

    let root = fixture("nextjs-coverage", "covered");
    let error = playwright_related_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap_err();
    assert!(error
        .reason
        .contains("files must contain at least one file"));
}

#[test]
fn tests_plan_json_ignores_deleted_changed_files() {
    let output = tests_plan_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "framework": "vitest",
            "root": fixture_root("test-plan-config"),
            "changedFiles": ["web/app/deleted.tsx", "source.ts"],
            "limitFiles": 1
        })
        .to_string(),
    ))
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["warnings"].as_array().unwrap().is_empty());
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert!(plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|test| test["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .all(|reason| { reason["changed_file"] != "web/app/deleted.tsx" })));
}

#[test]
fn queues_json_returns_project_report() {
    let options = json!({ "root": fixture_root("queue-dashboard/good") }).to_string();
    let output = queues_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["jobs"].as_array().unwrap().is_empty());
    assert!(value["diagnostics"].as_array().unwrap().is_empty());
}
