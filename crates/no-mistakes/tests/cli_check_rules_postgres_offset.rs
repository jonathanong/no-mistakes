use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-offset/fixture")
            .join(scenario),
    )
}

fn check(root: &PathBuf, yaml: &str) -> Output {
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), yaml).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap()
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    let yaml = std::fs::read_to_string(root.join(name)).unwrap();
    check(root, &yaml)
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

const RULE: &str = "postgres-no-offset";

#[test]
fn postgres_no_offset_fails_for_offset_clause() {
    let root = fixture("fail");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("query.ts"), "{body}");
    assert!(body.contains("OFFSET"), "{body}");
}

#[test]
fn postgres_no_offset_fails_for_interpolated_offset() {
    let root = fixture("fail-placeholder");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
}

#[test]
fn postgres_no_offset_passes_with_limit_only() {
    let root = fixture("pass-limit");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn parameter_builder_is_shared_by_annotation_and_offset_rules() {
    let root = fixture("parameter-builder");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config", ".no-mistakes.yml", "--json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(!out.status.success(), "expected findings: {body}");
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 4, "{body}");
    assert!(
        findings.iter().any(|f| f["line"] == 11
            && f["rule"] == "postgres-require-query-annotation"
            && f["target"] == "annotation"),
        "{body}"
    );
    assert!(
        findings.iter().any(|f| f["line"] == 16
            && f["rule"] == "postgres-no-offset"
            && f["target"] == "offset"),
        "{body}"
    );
    assert!(
        findings.iter().any(|f| f["line"] == 31
            && f["rule"] == "postgres-no-offset"
            && f["target"] == "unanalyzable"),
        "{body}"
    );
    assert!(
        findings.iter().any(|f| f["line"] == 40
            && f["rule"] == "postgres-no-offset"
            && f["target"] == "unanalyzable"),
        "{body}"
    );
}

#[test]
fn postgres_no_offset_passes_prose_offset() {
    let root = fixture("pass-prose");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_no_offset_json_has_rule_id() {
    let root = fixture("fail");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(body.contains(RULE), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_no_offset_flags_sql_files() {
    let root = fixture("fail-sql");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("optimizer fence"), "{body}");
    assert!(body.contains("do not use SQL OFFSET"), "{body}");
}

#[test]
fn postgres_no_offset_passes_materialized_cte() {
    let root = fixture("pass-sql");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_no_offset_ignores_sql_when_sql_include_is_empty() {
    let root = fixture("pass-default");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_no_offset_filesystem_runner_discovers_files() {
    let root = fixture("fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == no_mistakes::codebase::rules::POSTGRES_NO_OFFSET),
        "{body}"
    );
}
