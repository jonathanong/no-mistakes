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

// Framework-plan targeted tracing: a Playwright plan where a lockfile bump to
// `lodash` (which IS imported by src/utils.ts, which IS imported by e2e/utils.spec.ts)
// results in targeted selection of only that spec — not the unrelated.spec.ts.
// Before this change, any parseable lockfile diff unconditionally fell back to the full
// suite for framework plans; now BFS seeds from NodeId::Module("lodash") correctly
// narrows the selection.
#[test]
fn tests_plan_framework_playwright_lockfile_traceable_dep_selects_targeted_specs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    copy_dir_all(
        &fixture_dir("framework-plan-lockfile-traceable/initial"),
        root,
    );
    setup_git_repo(root);

    std::fs::copy(
        fixture_dir("framework-plan-lockfile-traceable/after-pnpm-lock.yaml"),
        root.join("pnpm-lock.yaml"),
    )
    .unwrap();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "playwright",
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
        "traceable package bump must not fall back: {plan:?}"
    );
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    assert!(
        selected.iter().any(|f| f.contains("utils.spec")),
        "utils.spec.ts must be selected (lodash → src/utils.ts → e2e/utils.spec.ts): {selected:?}"
    );
    assert!(
        !selected.iter().any(|f| f.contains("unrelated")),
        "unrelated.spec.ts must NOT be selected (no lodash import): {selected:?}"
    );
}

// Framework-plan untraceable fallback: a Playwright plan where the lockfile bumps
// `typescript` (a tooling dep with no import-graph path to any test spec) triggers
// a full-suite fallback only when --global-config-fallback=true is set.
// Without the flag no fallback fires and the plan has zero selected tests (the package
// is simply untraceable but not a hard error).
#[test]
fn tests_plan_framework_playwright_lockfile_untraceable_dep_falls_back_with_flag() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    copy_dir_all(
        &fixture_dir("framework-plan-lockfile-untraceable/initial"),
        root,
    );
    setup_git_repo(root);

    std::fs::copy(
        fixture_dir("framework-plan-lockfile-untraceable/after-pnpm-lock.yaml"),
        root.join("pnpm-lock.yaml"),
    )
    .unwrap();

    let output_with_flag = Command::new(bin())
        .args([
            "tests",
            "plan",
            "playwright",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--base",
            "HEAD",
            "--global-config-fallback=true",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output_with_flag.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output_with_flag.stderr)
    );
    let plan_with_flag: serde_json::Value =
        serde_json::from_str(&stdout(&output_with_flag)).unwrap();
    assert!(
        plan_with_flag["fallback_triggered"].as_bool().unwrap(),
        "untraceable tooling dep + --global-config-fallback must trigger fallback: {plan_with_flag:?}"
    );

    // Without the flag: no fallback, zero selected (untraceable but not forced).
    let output_no_flag = Command::new(bin())
        .args([
            "tests",
            "plan",
            "playwright",
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
        output_no_flag.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output_no_flag.stderr)
    );
    let plan_no_flag: serde_json::Value = serde_json::from_str(&stdout(&output_no_flag)).unwrap();
    assert!(
        !plan_no_flag["fallback_triggered"].as_bool().unwrap(),
        "untraceable tooling dep without flag must not force fallback: {plan_no_flag:?}"
    );
    let selected_no_flag = plan_no_flag["selected_tests"].as_array().unwrap();
    assert!(
        selected_no_flag.is_empty(),
        "no tests reachable from typescript dep — selected should be empty: {selected_no_flag:?}"
    );
}
