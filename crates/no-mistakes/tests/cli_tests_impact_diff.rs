use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

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
fn test_plan_diff_file_finds_impacted_tests() {
    let root = fixture("tests-impact-diff");
    let diff_path = root.join("sample.diff");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--diff",
        diff_path.to_str().unwrap(),
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "a.test.mts"),
        "should find a.test.mts via b.mts change: {selected:?}"
    );
}

#[test]
fn test_plan_diff_command_runs_and_parses() {
    let root = fixture("tests-impact-diff");
    let diff_path = root.join("sample.diff");
    let cmd = format!("cat {}", diff_path.to_str().unwrap());
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--diff-command",
        &cmd,
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "a.test.mts"),
        "should find a.test.mts: {selected:?}"
    );
}

#[test]
fn test_plan_diff_stdin_parses() {
    let root = fixture("tests-impact-diff");
    let diff_content = std::fs::read_to_string(root.join("sample.diff")).unwrap();

    let mut child = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--diff-stdin",
            "--json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "a.test.mts"),
        "should find a.test.mts: {selected:?}"
    );
}

#[test]
fn test_plan_diff_deleted_file_emits_warning() {
    let root = fixture("tests-impact-diff");
    let diff_path = root.join("delete.diff");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--diff",
        diff_path.to_str().unwrap(),
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"] == "deleted-file" && w["file"] == "deleted-route.mts"),
        "should have deleted-file warning: {warnings:?}"
    );
}

#[test]
fn test_plan_no_input_returns_empty() {
    let root = fixture("tests-impact-diff");
    let output = run(&["tests", "plan", "--root", root.to_str().unwrap(), "--json"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(selected.is_empty(), "no input should produce empty plan");
}

#[test]
fn test_plan_entrypoint_flag_works() {
    let root = fixture("tests-impact-diff");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--entrypoint",
        "c.mts",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "a.test.mts"),
        "entrypoint c.mts should find a.test.mts: {selected:?}"
    );
}

#[test]
fn test_plan_changed_file_explicit_still_works() {
    let root = fixture("tests-impact-diff");
    let output = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "c.mts",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "a.test.mts"),
        "changed-file c.mts should find a.test.mts: {selected:?}"
    );
}
