mod common;

use common::{fixture, run, stdout};

#[test]
fn test_plan_swift_native_source_uses_package_dependency_graph() {
    let root = fixture("swift-test-plan");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "swift-clients/core/Sources/VouchaCore/APIClient.swift",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec![
            "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift",
            "swift-clients/ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift",
        ]
    );

    let core_reason = &plan["selected_tests"][0]["reasons"][0];
    assert_eq!(
        core_reason["changed_file"],
        "swift-clients/core/Sources/VouchaCore/APIClient.swift"
    );
    assert_eq!(
        core_reason["path"],
        serde_json::json!([
            "swift-clients/core/Sources/VouchaCore/APIClient.swift",
            "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift"
        ])
    );
    assert_eq!(core_reason["via"], serde_json::json!(["swift"]));

    let ui_reason = &plan["selected_tests"][1]["reasons"][0];
    assert_eq!(
        ui_reason["changed_file"],
        "swift-clients/core/Sources/VouchaCore/APIClient.swift"
    );
    assert_eq!(
        ui_reason["path"],
        serde_json::json!([
            "swift-clients/core/Sources/VouchaCore/APIClient.swift",
            "swift-clients/ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift"
        ])
    );
    assert_eq!(
        ui_reason["via"],
        serde_json::json!(["swift package dependency"])
    );
}

#[test]
fn test_plan_swift_native_source_commands_format_uses_package_filters() {
    let root = fixture("swift-test-plan");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "swift-clients/core/Sources/VouchaCore/APIClient.swift",
        "--format",
        "commands",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = stdout(&output);
    let commands: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(
        commands,
        vec![
            "swift test --package-path swift-clients/core --filter VouchaCoreTests",
            "swift test --package-path swift-clients/ui --filter VouchaUITests",
        ]
    );
}
