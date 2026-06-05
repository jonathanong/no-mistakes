use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn lockfile_changed_file_without_base_emits_warning() {
    let root = fixture("lockfile-impact-no-baseline");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "pnpm-lock.yaml",
        "--global-config-fallback=true",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        plan["fallback_triggered"].as_bool().unwrap(),
        "should trigger fallback when no baseline: {plan:?}"
    );
    let reason = plan["fallback_reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("baseline") || reason.contains("lockfile"),
        "fallback_reason should mention baseline/lockfile: {reason}"
    );
}

#[test]
fn lockfile_changed_file_without_base_no_fallback_when_disabled() {
    let root = fixture("lockfile-impact-no-baseline");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "pnpm-lock.yaml",
        "--global-config-fallback=false",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "fallback should not trigger when disabled: {plan:?}"
    );
}

#[test]
fn lockfile_diff_subcommand_invalid_base_exits_ok_with_warning() {
    let root = fixture("lockfile-impact-pnpm");
    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        "nonexistent-branch-xyzzy",
        "--lockfile",
        "pnpm-lock.yaml",
    ]);
    assert!(
        output.status.success(),
        "should exit 0 even on missing base: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = stdout(&output);
    let arr: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        arr.as_array().unwrap().len(),
        0,
        "no diff when base unreachable"
    );
}

#[test]
fn lockfile_diff_subcommand_with_git_repo() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let old_lock = r#"lockfileVersion: '9.0'

packages:
  lodash@4.17.20:
    resolution: {integrity: sha512-PlhdFcillOINfeV7Ni6oF1TAEayyZBoZ8bcshTHqOYJYlrqzRK5hagpagky5o4HfCzzd1TRkXPMFq6cKk9rGg==}
"#;
    let new_lock = r#"lockfileVersion: '9.0'

packages:
  lodash@4.17.21:
    resolution: {integrity: sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZXGO6ysFxfwjE3d56g==}
"#;

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
        .args(["add", "pnpm-lock.yaml"])
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
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--lockfile",
            "pnpm-lock.yaml",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = stdout(&output);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["manager"], "pnpm");
    let changed = arr[0]["changed"].as_array().unwrap();
    assert!(
        changed.iter().any(|v| v == "lodash"),
        "should detect lodash as changed: {changed:?}"
    );
}

#[test]
fn lockfile_diff_paths_format() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let old_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.20:\n    resolution: {integrity: sha512-old}\n";
    let new_lock = "lockfileVersion: '9.0'\n\npackages:\n  lodash@4.17.21:\n    resolution: {integrity: sha512-new}\n";

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
        .args(["add", "pnpm-lock.yaml"])
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
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--lockfile",
            "pnpm-lock.yaml",
            "--format",
            "paths",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(
        out.contains("lodash"),
        "paths format should list lodash: {out}"
    );
}

#[test]
fn tests_plan_lockfile_no_global_config_trigger_with_lockfile() {
    let root = fixture("lockfile-impact-pnpm");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "pnpm-lock.yaml",
        "--global-config-fallback=true",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(
        plan["fallback_triggered"].as_bool().unwrap(),
        "should trigger fallback when no baseline: {plan:?}"
    );
    let reason = plan["fallback_reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("baseline") || reason.contains("lockfile"),
        "fallback_reason should mention baseline: {reason}"
    );
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
