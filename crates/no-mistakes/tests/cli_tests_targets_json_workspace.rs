use std::path::PathBuf;
use std::process::Command;

#[test]
fn tests_targets_cli_uses_workspace_for_json_project_arrays() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-workspace-json");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "inline/inline.test.ts",
            "string-project/string.test.ts",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "commands",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "vitest --workspace vitest.workspace.json --project json-inline inline/inline.test.ts"
    ));
    assert!(stdout.contains(
        "vitest --workspace vitest.projects.json --project json-string string-project/string.test.ts"
    ));
}

#[test]
fn tests_targets_cli_preserves_imported_concise_and_block_default_project_arrays() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-project-entries");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "concise/search.test.ts",
            "block/search.test.ts",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "commands",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "vitest --config vitest.config.ts --project imported-concise-arrow concise/search.test.ts"
    ));
    assert!(stdout.contains(
        "vitest --config vitest.config.ts --project imported-block-arrow block/search.test.ts"
    ));
}
