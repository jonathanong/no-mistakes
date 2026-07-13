use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-plan-lockfile")
        .join(name)
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()));
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
        }
    }
}

fn setup_git_repo(root: &std::path::Path) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    for (k, v) in [("user.email", "test@test.com"), ("user.name", "Test")] {
        Command::new("git")
            .args(["config", k, v])
            .current_dir(root)
            .output()
            .unwrap();
    }
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

// A binary lockfile cannot be parsed, but selecting the full suite remains an
// explicit policy choice through --global-config-fallback.
#[test]
fn tests_plan_binary_lockfile_fallback_requires_explicit_opt_in() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir_all(&fixture_dir("binary-lockfile-fallback"), root);
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "bun.lockb",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false, "{plan:?}");
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-binary-unsupported"),
        "expected lockfile-binary-unsupported warning: {plan:?}"
    );

    let opted_in = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "bun.lockb",
            "--global-config-fallback=true",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        opted_in.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&opted_in.stderr)
    );
    let opted_in: serde_json::Value = serde_json::from_str(&stdout(&opted_in)).unwrap();
    assert_eq!(opted_in["fallback_triggered"], true, "{opted_in:?}");
    assert_eq!(opted_in["selected_tests"].as_array().unwrap().len(), 1);
}

// Covers workspace package tracing: when a lockfile bumps a workspace package
// version, plan.rs falls back to NodeId::File(workspace_entry) when
// NodeId::Module(pkg_name) has no reverse edges, and BFS from the entry file
// reaches consumers of that workspace package.
#[test]
fn tests_plan_workspace_package_bump_traces_consumers() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Copy and commit the initial workspace layout (includes the "before" lockfile)
    copy_dir_all(&fixture_dir("workspace-package-bump/initial"), root);
    setup_git_repo(root);

    // Replace the lockfile with the "after" version on disk (uncommitted)
    std::fs::copy(
        fixture_dir("workspace-package-bump/after-pnpm-lock.yaml"),
        root.join("pnpm-lock.yaml"),
    )
    .unwrap();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "workspace package should be traceable, not fall back: {plan:?}"
    );
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected
            .iter()
            .any(|t| t["test_file"].as_str().unwrap().contains("utils.test")),
        "should trace workspace package lib to utils.test.ts: {selected:?}"
    );
}

// Covers analyze_lockfile_changes invalid-head path: when --head is a non-existent ref,
// git_ref_exists returns false and a lockfile-no-baseline warning is emitted rather
// than treating the new lockfile content as empty (which would falsely remove all packages).
#[test]
fn tests_plan_invalid_head_ref_warns_and_falls_back() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir_all(&fixture_dir("invalid-head-ref"), root);
    setup_git_repo(root);
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD",
            "--head",
            "nonexistent-head-xyz",
            "--global-config-fallback=true",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        plan["fallback_triggered"].as_bool().unwrap(),
        "invalid head should trigger fallback: {plan:?}"
    );
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-no-baseline"),
        "expected lockfile-no-baseline warning: {plan:?}"
    );
}
