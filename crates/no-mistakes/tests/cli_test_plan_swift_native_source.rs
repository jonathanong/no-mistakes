mod common;

use common::{fixture, run, stdout};

#[test]
fn test_plan_swift_native_source_uses_package_dependency_graph() {
    let root = fixture("../../fixtures/test-plan/swift-native-topology");
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
            "swift-clients/android/Tests/VouchaAndroidTests/AppTests.swift",
            "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift",
            "swift-clients/ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift",
        ]
    );

    let core_reason = &plan["selected_tests"][1]["reasons"][0];
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

    let ui_reason = &plan["selected_tests"][2]["reasons"][0];
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
fn test_plan_swift_android_source_stays_in_android_package() {
    let root = fixture("../../fixtures/test-plan/swift-native-topology");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "swift-clients/android/Sources/VouchaAndroid/App.swift",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec!["swift-clients/android/Tests/VouchaAndroidTests/AppTests.swift"]
    );
}

#[test]
fn test_plan_swift_native_source_commands_format_uses_package_filters() {
    let root = fixture("../../fixtures/test-plan/swift-native-topology");
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
            "swift test --package-path swift-clients/android --filter VouchaAndroidTests",
            "swift test --package-path swift-clients/core --filter VouchaCoreTests",
            "swift test --package-path swift-clients/ui --filter VouchaUITests",
        ]
    );
}

#[test]
fn test_plan_swift_include_glob_accounts_and_targets_only_selected_package() {
    let root = fixture("swift-test-plan");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "backend/api/feeds.mts",
        "--environment",
        "all",
        "--include-glob",
        "swift-clients/apps/android/**",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0]["test_file"],
        "swift-clients/apps/android/Tests/VouchaAndroidTests/DeviceTests.swift"
    );
    assert!(selected[0]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|target| {
            target["config"] == "swift-clients/apps/android"
                && target["project"] == "VouchaAndroidTests"
                && target["runner_args"]
                    == serde_json::json!([
                        "--package-path",
                        "swift-clients/apps/android",
                        "--filter",
                        "VouchaAndroidTests"
                    ])
        }));
    assert_eq!(
        plan["groups"],
        serde_json::json!([{
            "type": "all",
            "selected": ["swift-clients/apps/android/Tests/VouchaAndroidTests/DeviceTests.swift"],
            "remaining": 0,
            "limit": null
        }])
    );
    assert!(plan["execution_targets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|target| {
            target["runner"] == "swift"
                && target["config"] == "swift-clients/apps/android"
                && target["project"] == "VouchaAndroidTests"
                && target["name"] == "swift-clients/apps/android"
                && target["runner_args"]
                    == serde_json::json!([
                        "--package-path",
                        "swift-clients/apps/android",
                        "--filter",
                        "VouchaAndroidTests"
                    ])
                && target["test_files"]
                    == serde_json::json!([
                        "swift-clients/apps/android/Tests/VouchaAndroidTests/DeviceTests.swift"
                    ])
        }));
}
