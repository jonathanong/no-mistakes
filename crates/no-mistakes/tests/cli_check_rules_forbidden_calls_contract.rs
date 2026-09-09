#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;

use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn check(root: &Path, config: &Path) -> (bool, String, Value) {
    let output = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let report = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    (output.status.success(), format!("{stdout}{stderr}"), report)
}

fn rules(report: &Value) -> &[Value] {
    report["rules"].as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn config(root: &Path, name: &str) -> std::path::PathBuf {
    root.join("configs").join(name)
}

#[test]
fn named_vitest_project_does_not_select_integration_files() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (_, _, report) = check(root.path(), &config(root.path(), "named-unit.yml"));
    let findings = rules(&report);
    assert!(findings
        .iter()
        .any(|finding| finding["file"] == "src/unit/timers.test.mts"));
    assert!(!findings
        .iter()
        .any(|finding| finding["file"] == "src/integration/timers.test.mts"));
}

#[test]
fn exclude_filters_findings_without_changing_root_expansion() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (_, _, report) = check(root.path(), &config(root.path(), "exclude-integration.yml"));
    let findings = rules(&report);
    assert!(findings
        .iter()
        .any(|finding| finding["file"] == "src/unit/timers.test.mts"));
    assert!(!findings
        .iter()
        .any(|finding| finding["file"] == "src/integration/timers.test.mts"));
}

#[test]
fn same_name_explicit_project_replaces_runner_include() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (_, _, report) = check(root.path(), &config(root.path(), "overlap-unit.yml"));
    let files: Vec<_> = rules(&report)
        .iter()
        .map(|finding| finding["file"].as_str().unwrap())
        .collect();
    assert!(files.contains(&"src/unit/timers.test.mts"), "{report:#?}");
    assert!(
        !files.iter().any(|file| file.contains("allowed.test")),
        "{report:#?}"
    );
}

#[test]
fn explicit_only_projects_union_without_runner_config() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (ok, body, report) = check(root.path(), &config(root.path(), "explicit-only.yml"));
    assert!(!ok, "{body}");
    assert!(rules(&report)
        .iter()
        .any(|finding| finding["file"] == "src/unit/timers.test.mts"));
}

#[test]
fn unknown_vitest_project_fails_closed() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (ok, body, _) = check(root.path(), &config(root.path(), "unknown-project.yml"));
    assert!(!ok, "{body}");
    assert!(body.contains("names an unknown project"), "{body}");
}

#[test]
fn malformed_selected_root_fails_closed() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (ok, body, _) = check(root.path(), &config(root.path(), "malformed-root.yml"));
    assert!(!ok, "{body}");
    assert!(body.contains("failed to parse"), "{body}");
    assert!(body.contains("src/broken.mts"), "{body}");
}

#[test]
fn suppressions_do_not_hide_configuration_errors() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (ok, body, _) = check(root.path(), &config(root.path(), "config-error.yml"));
    assert!(!ok, "{body}");
    assert!(
        body.contains("does-not-exist") || body.contains("configured root"),
        "{body}"
    );
}

#[test]
fn unknown_calls_finding_reports_dynamic_access() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (_, _, report) = check(
        root.path(),
        &config(root.path(), "unknown-calls-finding.yml"),
    );
    assert!(
        rules(&report).iter().any(|finding| {
            finding["file"] == "src/unit/dynamic.test.mts" && finding["target"] == "unknown call"
        }),
        "{report:#?}"
    );
}

#[test]
fn exact_selector_matches_unresolved_page_wait_for_timeout() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (_, _, report) = check(root.path(), &config(root.path(), "exact-unknown.yml"));
    let matches = rules(&report)
        .iter()
        .filter(|finding| finding["import"] == "page.waitForTimeout")
        .count();
    assert!(matches >= 2, "{report:#?}");
}

#[test]
fn retired_forbidden_calls_option_fails_closed() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (ok, body, _) = check(
        root.path(),
        &config(root.path(), "retired-forbidden-calls.yml"),
    );
    assert!(!ok, "{body}");
    assert!(body.contains("forbiddenCalls"), "{body}");
    assert!(body.contains("forbidden-calls"), "{body}");
}
