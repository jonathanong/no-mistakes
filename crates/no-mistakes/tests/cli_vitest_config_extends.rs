use std::path::PathBuf;
use std::process::Command;

#[test]
fn tests_targets_cli_uses_inline_static_vitest_config_extends() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-extends-config");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "extended/owned.test.ts",
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
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("vitest --config vitest.config.ts --project extended extended/owned.test.ts"));
}

#[test]
fn tests_plan_cli_traces_inline_static_vitest_config_extends() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-extends-config");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "plan",
            "vitest",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "base-setup.ts",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|test| test["test_file"] == "extended/owned.test.ts"));
}

#[test]
fn tests_targets_cli_uses_inherited_inline_vitest_scope() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-extends-config");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "scope-inherited/owned/owned.spec.ts",
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
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("vitest --config vitest.config.ts --project scope-inherited"));
}

#[test]
fn tests_targets_cli_keeps_local_inline_scope_base() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-extends-config");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "local-root/local/owned.test.ts",
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
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("vitest --config vitest.config.ts --project cross-local"));
}

#[test]
fn tests_targets_cli_uses_merged_inline_vitest_extends() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-extends-config");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "merged-root/owned/merged.test.ts",
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
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("vitest --config vitest.config.ts --project merged-extends"));
}
