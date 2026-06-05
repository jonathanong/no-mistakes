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

#[test]
fn tests_plan_with_lockfile_base_finds_impacted_tests() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let old_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let new_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";

    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"t","dependencies":{"lodash":"^4.17.0"}}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/utils.mts"),
        "import { pick } from \"lodash\";\nexport const utils = { pick };\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/utils.test.mts"),
        "import { utils } from \"./utils.mts\";\n",
    )
    .unwrap();
    std::fs::write(root.join("pnpm-lock.yaml"), old_lock).unwrap();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();
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
        "should not trigger fallback: {plan:?}"
    );
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected
            .iter()
            .any(|t| t["test_file"].as_str().unwrap().contains("utils.test")),
        "should find utils.test.mts via lodash change: {selected:?}"
    );
}

// Covers the --head branch in analyze_lockfile_changes: new content from git show <head>:.
#[test]
fn tests_plan_lockfile_head_option() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let old_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let new_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"t","dependencies":{"lodash":"^4.0.0"}}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/utils.mts"),
        "import { pick } from \"lodash\";\nexport const u = pick;\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/utils.test.mts"),
        "import { u } from \"./utils.mts\";\n",
    )
    .unwrap();
    setup_git_repo_with_file(root, "pnpm-lock.yaml", old_lock);
    std::fs::write(root.join("pnpm-lock.yaml"), new_lock).unwrap();
    Command::new("git")
        .args(["add", "pnpm-lock.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "v2"])
        .current_dir(root)
        .output()
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
            "HEAD~1",
            "--head",
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
        "should not fallback: {plan:?}"
    );
}

// Covers find_git_root returning None (tempdir without git init): no-baseline warning emitted
// because args.base is None, so analyze_lockfile_changes falls through to the None branch.
// find_git_root is always called but returns None here; git_root falls back to root.
#[test]
fn tests_plan_lockfile_no_git_repo_emits_warning() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
    // No --base: avoids the CLI's own git-diff phase (which would fail in a non-git dir).
    // analyze_lockfile_changes still calls find_git_root (returns None → git_root = root)
    // and then hits the args.base = None branch, emitting the lockfile-no-baseline warning.
    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"].as_str().unwrap_or("") == "lockfile-no-baseline"),
        "expected lockfile-no-baseline warning: {plan:?}"
    );
}
