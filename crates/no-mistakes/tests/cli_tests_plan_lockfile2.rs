use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn setup_git_repo_with_file(root: &std::path::Path, filename: &str, content: &str) {
    std::fs::write(root.join(filename), content).unwrap();
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
        .args(["add", filename])
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

// Covers the binary_lockfile_fallback path: bun.lockb is a binary lockfile that
// cannot be parsed. The fallback is unconditional (no --global-config-fallback flag
// required) because a lockfile change was detected but cannot be analysed, so
// zero selected tests would be incorrect.
#[test]
fn tests_plan_binary_lockfile_fallback_is_unconditional() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir(root.join("src")).unwrap();
    // bun.lockb is a binary lockfile — write any bytes as a stand-in
    std::fs::write(root.join("bun.lockb"), b"binarydata").unwrap();
    std::fs::write(root.join("src/utils.test.ts"), "export {};\n").unwrap();
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "bun.lockb",
            "--json",
            // Deliberately omit --global-config-fallback to verify it is not needed
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
        plan["fallback_triggered"].as_bool().unwrap(),
        "binary lockfile fallback must trigger without --global-config-fallback: {plan:?}"
    );
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-binary-unsupported"),
        "expected lockfile-binary-unsupported warning: {plan:?}"
    );
}

// Covers workspace package tracing: when a lockfile bumps a workspace package
// version, plan.rs falls back to NodeId::File(workspace_entry) when
// NodeId::Module(pkg_name) has no reverse edges, and BFS from the entry file
// reaches consumers of that workspace package.
#[test]
fn tests_plan_workspace_package_bump_traces_consumers() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("pnpm-workspace.yaml"),
        "packages:\n  - packages/*\n",
    )
    .unwrap();

    // Workspace package: lib
    std::fs::create_dir_all(root.join("packages/lib/src")).unwrap();
    std::fs::write(
        root.join("packages/lib/package.json"),
        r#"{"name":"lib","main":"src/index.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("packages/lib/src/index.ts"),
        "export const foo = 1;\n",
    )
    .unwrap();

    // Workspace package: app (consumer of lib)
    std::fs::create_dir_all(root.join("packages/app/src")).unwrap();
    std::fs::write(
        root.join("packages/app/package.json"),
        r#"{"name":"app","dependencies":{"lib":"workspace:*"}}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("packages/app/src/utils.ts"),
        "import { foo } from 'lib';\nexport { foo };\n",
    )
    .unwrap();
    std::fs::write(
        root.join("packages/app/src/utils.test.ts"),
        "import { foo } from './utils.ts';\n",
    )
    .unwrap();

    // Commit old lockfile at base
    let old_lock = "lockfileVersion: '9.0'\n\npackages:\n  lib@1.0.0:\n    resolution: {directory: packages/lib, type: directory}\n";
    setup_git_repo_with_file(root, "pnpm-lock.yaml", old_lock);

    // Bump lib version on disk
    let new_lock = "lockfileVersion: '9.0'\n\npackages:\n  lib@1.1.0:\n    resolution: {directory: packages/lib, type: directory}\n";
    std::fs::write(root.join("pnpm-lock.yaml"), new_lock).unwrap();

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
    let lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-x}\n";
    setup_git_repo_with_file(root, "pnpm-lock.yaml", lock);
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
