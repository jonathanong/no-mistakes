use std::path::Path;

use crate::common;

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
        "swift-clients/core/Sources/ExampleCore/APIClient.swift",
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
            "swift-clients/android/Tests/ExampleAndroidTests/AppTests.swift",
            "swift-clients/core/Tests/ExampleCoreTests/APIClientTests.swift",
            "swift-clients/ui/Tests/ExampleUITests/RSSFeedListViewModelTests.swift",
        ]
    );

    let core_reason = &plan["selected_tests"][1]["reasons"][0];
    assert_eq!(
        core_reason["changed_file"],
        "swift-clients/core/Sources/ExampleCore/APIClient.swift"
    );
    assert_eq!(
        core_reason["path"],
        serde_json::json!([
            "swift-clients/core/Sources/ExampleCore/APIClient.swift",
            "swift-clients/core/Tests/ExampleCoreTests/APIClientTests.swift"
        ])
    );
    assert_eq!(core_reason["via"], serde_json::json!(["swift"]));

    let ui_reason = &plan["selected_tests"][2]["reasons"][0];
    assert_eq!(
        ui_reason["changed_file"],
        "swift-clients/core/Sources/ExampleCore/APIClient.swift"
    );
    assert_eq!(
        ui_reason["path"],
        serde_json::json!([
            "swift-clients/core/Sources/ExampleCore/APIClient.swift",
            "swift-clients/ui/Tests/ExampleUITests/RSSFeedListViewModelTests.swift"
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
        "swift-clients/android/Sources/ExampleAndroid/App.swift",
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
        vec!["swift-clients/android/Tests/ExampleAndroidTests/AppTests.swift"]
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
        "swift-clients/core/Sources/ExampleCore/APIClient.swift",
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
    // `.no-mistakes.yml` omits `tests.swift.baseCommands`. This list is the
    // byte-identical default: `swift test` plus generated package and filter args.
    assert_eq!(
        commands,
        vec![
            "swift test --package-path swift-clients/android --filter ExampleAndroidTests",
            "swift test --package-path swift-clients/core --filter ExampleCoreTests",
            "swift test --package-path swift-clients/ui --filter ExampleUITests",
        ]
    );
}

#[test]
fn test_plan_swift_base_commands_override_only_configured_packages() {
    let root = fixture("../../fixtures/test-plan/swift-native-topology");
    let config = root.join("runner-argv.no-mistakes.yml");
    let output = run(&[
        "test",
        "plan",
        "swift",
        "--root",
        root.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--changed-file",
        "swift-clients/core/Sources/ExampleCore/APIClient.swift",
        "--format",
        "commands",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = stdout(&output);
    let commands: Vec<&str> = rendered.trim().lines().collect();
    assert_eq!(
        commands,
        vec![
            "'/opt/Swift 6.3/usr/bin/swift' test --package-path swift-clients/ui --filter ExampleUITests",
            "'host'\\''s swift' test --package-path swift-clients/android --filter ExampleAndroidTests",
            "swift test --package-path swift-clients/core --filter ExampleCoreTests",
        ]
    );
}

#[test]
fn test_targets_swift_base_commands_keep_generated_filters() {
    let root = fixture("../../fixtures/test-plan/swift-native-topology");
    let config = root.join("runner-argv.no-mistakes.yml");
    let output = run(&[
        "tests",
        "targets",
        "swift",
        "swift-clients/android/Tests/ExampleAndroidTests/AppTests.swift",
        "swift-clients/core/Tests/ExampleCoreTests/APIClientTests.swift",
        "--root",
        root.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--format",
        "commands",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = stdout(&output);
    let commands: Vec<&str> = rendered.trim().lines().collect();
    assert_eq!(
        commands,
        vec![
            "'host'\\''s swift' test --package-path swift-clients/android --filter ExampleAndroidTests",
            "swift test --package-path swift-clients/core --filter ExampleCoreTests",
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
        "swift-clients/apps/android/Tests/ExampleAndroidTests/DeviceTests.swift"
    );
    assert!(selected[0]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|target| {
            target["config"] == "swift-clients/apps/android"
                && target["project"] == "ExampleAndroidTests"
                && target["runner_args"]
                    == serde_json::json!([
                        "--package-path",
                        "swift-clients/apps/android",
                        "--filter",
                        "ExampleAndroidTests"
                    ])
        }));
    assert_eq!(
        plan["groups"],
        serde_json::json!([{
            "type": "all",
            "selected": ["swift-clients/apps/android/Tests/ExampleAndroidTests/DeviceTests.swift"],
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
                && target["project"] == "ExampleAndroidTests"
                && target["name"] == "swift-clients/apps/android"
                && target["runner_args"]
                    == serde_json::json!([
                        "--package-path",
                        "swift-clients/apps/android",
                        "--filter",
                        "ExampleAndroidTests"
                    ])
                && target["test_files"]
                    == serde_json::json!([
                        "swift-clients/apps/android/Tests/ExampleAndroidTests/DeviceTests.swift"
                    ])
        }));
}

#[test]
fn swift_base_commands_change_only_base_command() {
    let root = fixture("../../fixtures/test-plan/swift-native-topology");
    let changed = [
        "--changed-file",
        "swift-clients/core/Sources/ExampleCore/APIClient.swift",
    ];
    let plain = plan_swift_json(&root, None, &changed);
    let overridden = plan_swift_json(
        &root,
        Some(&root.join("runner-argv.no-mistakes.yml")),
        &changed,
    );
    assert_ne!(plain, overridden);
    assert_eq!(
        without_base_command(&plain),
        without_base_command(&overridden)
    );

    let diff = root.join("delete-core-package.diff");
    let diff_args = ["--diff", diff.to_str().unwrap()];
    let fallback = plan_swift_json(&root, None, &diff_args);
    let fallback_override = plan_swift_json(
        &root,
        Some(&root.join("runner-argv-core.no-mistakes.yml")),
        &diff_args,
    );
    assert_eq!(fallback["fallback_triggered"], true);
    assert_eq!(fallback_override["fallback_triggered"], true);
    assert_ne!(fallback, fallback_override);
    assert_eq!(
        without_base_command(&fallback),
        without_base_command(&fallback_override)
    );
}

#[test]
fn swift_base_commands_planning_does_not_execute_argv() {
    let root = fixture("../../fixtures/test-plan/swift-native-topology");
    let sentinel = root.join("sentinel-should-not-exist");
    assert!(!sentinel.exists());
    let plan = plan_swift_json(
        &root,
        Some(&root.join("runner-argv-sentinel.no-mistakes.yml")),
        &[
            "--changed-file",
            "swift-clients/core/Sources/ExampleCore/APIClient.swift",
        ],
    );
    assert!(plan.to_string().contains("sentinel-should-not-exist"));
    assert!(
        !sentinel.exists(),
        "planning must not create {}",
        sentinel.display()
    );
}

fn plan_swift_json(root: &Path, config: Option<&Path>, extra: &[&str]) -> serde_json::Value {
    let mut args = vec![
        "test".to_string(),
        "plan".to_string(),
        "swift".to_string(),
        "--root".to_string(),
        root.display().to_string(),
        "--json".to_string(),
    ];
    if let Some(config) = config {
        args.push("--config".to_string());
        args.push(config.display().to_string());
    }
    args.extend(extra.iter().map(|arg| (*arg).to_string()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run(&refs);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(&output)).unwrap()
}

fn without_base_command(value: &serde_json::Value) -> serde_json::Value {
    let mut value = value.clone();
    strip_base_command(&mut value);
    value
}

fn strip_base_command(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("base_command");
            for child in map.values_mut() {
                strip_base_command(child);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                strip_base_command(item);
            }
        }
        _ => {}
    }
}
