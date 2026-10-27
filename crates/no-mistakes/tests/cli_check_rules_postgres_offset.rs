use std::path::{Path, PathBuf};
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

fn config(rule: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("config")
        .join(name)
}

fn check(root: &Path, config: &Path) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

const RULE: &str = "postgres-no-offset";
const OFFSET: &str = "postgres_no_offset_fails_for_offset_clause.yml";
const OFFSET_SQL: &str = "postgres_no_offset_flags_sql_files.yml";

#[test]
fn postgres_no_offset_fails_for_offset_clause() {
    let root = fixture("fail");
    let out = check(&root, &config(RULE, OFFSET));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("query.ts"), "{body}");
    assert!(body.contains("OFFSET"), "{body}");
}

#[test]
fn postgres_no_offset_fails_for_interpolated_offset() {
    let root = fixture("fail-placeholder");
    let out = check(&root, &config(RULE, OFFSET));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
}

#[test]
fn postgres_no_offset_passes_with_limit_only() {
    let root = fixture("pass-limit");
    let out = check(&root, &config(RULE, OFFSET));
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
        // A literal tail (line 31) is recovered; a non-literal one is not.
        findings.iter().any(|f| f["line"] == 42
            && f["rule"] == "postgres-no-offset"
            && f["target"] == "unanalyzable"),
        "{body}"
    );
    assert!(findings.iter().all(|f| f["line"] != 31), "{body}");
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
    let out = check(&root, &config(RULE, OFFSET));
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
    let out = check(&root, &config(RULE, OFFSET_SQL));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("optimizer fence"), "{body}");
    assert!(body.contains("do not use SQL OFFSET"), "{body}");
}

#[test]
fn postgres_no_offset_passes_materialized_cte() {
    let root = fixture("pass-sql");
    let out = check(&root, &config(RULE, OFFSET_SQL));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_no_offset_ignores_sql_when_sql_include_is_empty() {
    let root = fixture("pass-default");
    let out = check(&root, &config(RULE, OFFSET));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

fn const_splice_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../fixtures/postgres/no-offset/const-splice/{name}"
    ))
}

fn offset_lines(root: &PathBuf) -> Vec<u64> {
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    let report: serde_json::Value =
        serde_json::from_str(&body).unwrap_or_else(|_| panic!("{body}"));
    report["rules"]
        .as_array()
        .unwrap_or_else(|| panic!("{body}"))
        .iter()
        .filter(|finding| {
            finding["rule"] == RULE
                && finding["target"]
                    .as_str()
                    .is_some_and(|target| target == "offset" || target.starts_with("offset#"))
        })
        .filter_map(|finding| finding["line"].as_u64())
        .collect()
}

#[test]
fn const_splice_ternary_reports_offset_on_the_call() {
    let lines = offset_lines(&const_splice_root("ternary"));
    assert_eq!(lines, vec![6], "{lines:?}");
}

#[test]
fn const_splice_poisoned_member_still_reports_offset_when_unanalyzable_is_ignored() {
    let lines = offset_lines(&const_splice_root("poison"));
    assert_eq!(lines, vec![6], "{lines:?}");
}

#[test]
fn const_splice_scalar_and_member_report_offset() {
    let mut lines = offset_lines(&const_splice_root("scalar"));
    lines.sort_unstable();
    assert_eq!(lines, vec![7, 11], "{lines:?}");
}

fn offset_findings(root: &PathBuf) -> Vec<serde_json::Value> {
    json_findings(root)
        .into_iter()
        .filter(|finding| {
            finding["rule"] == RULE
                && finding["target"]
                    .as_str()
                    .is_some_and(|target| target == "offset" || target.starts_with("offset#"))
        })
        .collect()
}

#[test]
fn const_splice_origins_reports_one_offset_on_the_call_line() {
    let findings = json_findings(&fixture("const-splice-origins"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0]["rule"], RULE);
    assert_eq!(findings[0]["file"], "src/query.ts");
    assert_eq!(findings[0]["line"], 5);
    assert_eq!(findings[0]["target"], "offset");
}

#[test]
fn const_splice_origins_cross_file_still_reports_the_other_offset() {
    let root = fixture("const-splice-origins-cross-file");
    let findings = offset_findings(&root);
    assert!(
        findings.iter().any(|finding| {
            finding["file"] == "src/paged.ts"
                && finding["line"] == 4
                && finding["target"] == "offset"
        }),
        "{findings:?}"
    );
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
fn json_findings(root: &PathBuf) -> Vec<serde_json::Value> {
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .args(["--config", ".no-mistakes.yml", "--json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(!out.status.success(), "expected findings: {body}");
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mut findings = report["rules"].as_array().cloned().unwrap_or_default();
    findings.sort_by(|left, right| {
        (
            left["rule"].as_str(),
            left["file"].as_str(),
            left["line"].as_u64(),
            left["target"].as_str(),
        )
            .cmp(&(
                right["rule"].as_str(),
                right["file"].as_str(),
                right["line"].as_u64(),
                right["target"].as_str(),
            ))
    });
    findings
}

#[test]
fn project_scope_matches_duplicated_executor_options() {
    let shared = json_findings(&fixture("project-scope-shared"));
    let duplicated = json_findings(&fixture("project-scope-duplicated"));
    assert!(!shared.is_empty(), "{shared:?}");
    assert_eq!(shared, duplicated);
    assert!(shared.iter().any(|finding| {
        finding["rule"] == "postgres-no-offset" && finding["file"] == "src/query.ts"
    }));
}

#[test]
fn project_include_skips_executor_calls_outside_the_project() {
    let findings = json_findings(&fixture("project-scope-include"));
    assert!(findings.iter().any(|finding| {
        finding["rule"] == "postgres-no-offset" && finding["file"] == "inside/query.ts"
    }));
    assert!(findings
        .iter()
        .all(|finding| finding["file"] != "outside/query.ts"));
}

#[path = "cli_check_rules_postgres_offset/builder_mutation_escapes.rs"]
mod builder_mutation_escapes;

#[path = "cli_check_rules_postgres_offset/effort.rs"]
mod effort;
