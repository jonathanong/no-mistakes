use std::path::PathBuf;

fn root() -> PathBuf {
    crate::check_json_snapshot::rules_path(
        "test-no-live-repository-scan",
        "owned-temporary-roots",
        "",
    )
}

#[test]
fn owned_temporary_roots_check_json_snapshot() {
    let root = root();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut files: Vec<_> = ["rules", "advisories"]
        .into_iter()
        .flat_map(|key| report[key].as_array().unwrap())
        .map(|finding| finding["file"].as_str().unwrap())
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        [
            "tests/async-unawaited.test.mts",
            "tests/await-non-call.test.mts",
            "tests/await-other-member.test.mts",
            "tests/default-async-alias-escape.test.mts",
            "tests/default-async-alias-mutation.test.mts",
            "tests/default-sync-alias-escape.test.mts",
            "tests/default-sync-alias-mutation.test.mts",
            "tests/escape-absolute.test.mts",
            "tests/escape-parent.test.mts",
            "tests/factory-callee-call.test.mts",
            "tests/factory-global.test.mjs",
            "tests/factory-receiver-call.test.mts",
            "tests/helper-absolute-return.test.mts",
            "tests/helper-buffer-return.test.mts",
            "tests/helper-empty-return.test.mts",
            "tests/helper-fallback.test.mts",
            "tests/helper-fallthrough.test.mts",
            "tests/helper-finally.test.mts",
            "tests/helper-live-branch.test.mts",
            "tests/helper-mutable.test.mts",
            "tests/helper-reassigned.test.mts",
            "tests/helper-recursive.test.mts",
            "tests/namespace-delete.test.mts",
            "tests/namespace-escape.test.mts",
            "tests/namespace-mutation.test.mts",
            "tests/namespace-valueof.test.mts",
            "tests/other-member.test.mts",
            "tests/raw-resource.test.mts",
            "tests/resource-escape.test.mts",
            "tests/shadow-factory.test.mts",
            "tests/sync-buffer-path.test.mts",
            "tests/wrong-export.test.mts",
            "tests/wrong-module.test.mts",
        ],
        "{report}"
    );
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_owned_temporary_roots",
        &[(
            "owned-temporary-roots",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}

#[test]
fn callback_roots_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "test-no-live-repository-scan",
        "callback-roots",
        "",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut files: Vec<_> = ["rules", "advisories"]
        .into_iter()
        .flat_map(|key| report[key].as_array().unwrap())
        .map(|finding| finding["file"].as_str().unwrap())
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        [
            "src/after-cleanup.mts",
            "src/async-unawaited.mts",
            "src/await-using-unawaited.mts",
            "src/buffer-root.mts",
            "src/callback-alias.mts",
            "src/callback-call.mts",
            "src/callback-escape.mts",
            "src/callback-factory.mts",
            "src/conditional-cleanup.mts",
            "src/escape.mts",
            "src/helper-reassignment.mts",
            "src/missing-cleanup.mts",
            "src/mixed-invocations.mts",
            "src/nested-cleanup.mts",
            "src/non-owned.mts",
            "src/nonrecursive-cleanup.mts",
            "src/parameter-reassignment.mts",
            "src/resource-escape.mts",
            "src/resource-no-using.mts",
            "src/returned-promise.mts",
            "src/root-return.mts",
            "src/unawaited-promise.mts",
            "src/unknown-sink.mts",
            "src/using-async.mts",
            "src/wrong-cleanup.mts",
        ],
        "{report}"
    );
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_callback_roots",
        &[(
            "callback-roots",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}

#[test]
fn callback_captures_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "test-no-live-repository-scan",
        "callback-captures",
        "",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut files: Vec<_> = ["rules", "advisories"]
        .into_iter()
        .flat_map(|key| report[key].as_array().unwrap())
        .map(|finding| finding["file"].as_str().unwrap())
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        [
            "src/non-owned-helper.mts",
            "src/reassigned-root.mts",
            "src/shadowed-root.mts",
        ],
        "{report}"
    );
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_callback_captures",
        &[(
            "callback-captures",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}

#[test]
fn callback_spreads_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "test-no-live-repository-scan",
        "callback-spreads",
        "",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let files: Vec<_> = ["rules", "advisories"]
        .into_iter()
        .flat_map(|key| report[key].as_array().unwrap())
        .map(|finding| finding["file"].as_str().unwrap())
        .collect();
    assert_eq!(files, ["src/spread-before-root.mts"], "{report}");
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_callback_spreads",
        &[(
            "callback-spreads",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}

#[test]
fn temp_factory_return_kinds_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "test-no-live-repository-scan",
        "temp-factory-return-kinds",
        "",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut enumerations: Vec<_> = ["rules", "advisories"]
        .into_iter()
        .flat_map(|key| report[key].as_array().unwrap())
        .filter(|finding| finding["import"] == "readdirSync")
        .map(|finding| {
            (
                finding["file"].as_str().unwrap().to_string(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    enumerations.sort_unstable();
    assert_eq!(
        enumerations,
        [
            ("tests/callback-factory.test.mts".to_string(), 7),
            ("tests/callback-one-argument.test.mts".to_string(), 7),
            ("tests/member-unawaited.test.mts".to_string(), 9),
            ("tests/member-unawaited.test.mts".to_string(), 10),
            ("tests/require-callback.test.cts".to_string(), 9),
            ("tests/require-callback.test.cts".to_string(), 10),
            ("tests/require-unawaited.test.cts".to_string(), 10),
            ("tests/require-unawaited.test.cts".to_string(), 11),
            ("tests/unawaited-promise.test.mts".to_string(), 8),
        ],
        "{report}"
    );
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_temp_factory_return_kinds",
        &[(
            "temp-factory-return-kinds",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}
