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
