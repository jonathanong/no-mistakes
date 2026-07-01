mod common;

use common::{fixture, run, stdout};

#[test]
fn test_plan_swift_falls_back_when_source_graph_is_unconfigured() {
    let root = fixture("swift-test-plan").join("swift-clients");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "core/Sources/VouchaAPI/Endpoint.swift",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("swift source impact"));

    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec![
            "core/Tests/VouchaCoreTests/APIClientTests.swift",
            "ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift",
        ]
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["base_command"],
        serde_json::json!(["swift", "test"])
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["runner_args"],
        serde_json::json!(["--filter", "VouchaCoreTests"])
    );
}

#[test]
fn test_plan_swift_scopes_package_manifest_fallback_to_package_tests() {
    let root = fixture("swift-test-plan");
    let config = root.join("no-trigger.no-mistakes.yml");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--changed-file",
        "swift-clients/core/Package.swift",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec!["swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift"]
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["config"],
        "swift-clients/core"
    );
}
