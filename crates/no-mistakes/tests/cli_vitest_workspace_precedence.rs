use std::path::PathBuf;
use std::process::Command;

#[test]
fn tests_targets_cli_prefers_default_vitest_workspace_ownership() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-workspace-precedence");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "workspace/owned.test.ts",
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
    assert!(String::from_utf8(output.stdout).unwrap().contains(
        "vitest --workspace vitest.workspace.ts --project workspace workspace/owned.test.ts"
    ));

    let root_config = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "root/root.test.ts",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "commands",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(root_config.status.success());
    assert!(!String::from_utf8(root_config.stdout)
        .unwrap()
        .contains("--config vitest.config.ts"));
}
