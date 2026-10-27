use std::path::PathBuf;
use std::process::{Command, Output};

fn root(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2")
        .join(scenario)
        .join("fixture")
}

fn check(scenario: &str) -> (Output, serde_json::Value) {
    check_with(scenario, &[])
}

fn check_with(scenario: &str, extra: &[&str]) -> (Output, serde_json::Value) {
    let root = root(scenario);
    let out = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .args(extra)
        .output()
        .unwrap();
    let report = serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);
    (out, report)
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn len(report: &serde_json::Value, key: &str) -> usize {
    report[key].as_array().map_or(0, Vec::len)
}

#[test]
fn unknown_rule_key_is_a_config_error_naming_key_rule_and_line() {
    let (out, _) = check("rule-severity-unknown-key");
    let err = stderr(&out);
    assert!(!out.status.success(), "{err}");
    assert!(err.contains(".no-mistakes.yml:4:"), "{err}");
    assert!(
        err.contains("rules[0] (rule `no-empty-or-comments-only-files`)"),
        "{err}"
    );
    assert!(err.contains("unknown key `serverity`"), "{err}");
    assert!(err.contains("Supported keys: name, rule, message"), "{err}");
    assert!(err.contains("severity"), "{err}");
}

#[test]
fn bogus_severity_lists_the_supported_values() {
    let (out, _) = check("rule-severity-bogus");
    let err = stderr(&out);
    assert!(!out.status.success(), "{err}");
    assert!(err.contains("bogus"), "{err}");
    assert!(
        err.contains("`error`") && err.contains("`advisory`"),
        "{err}"
    );
}

#[test]
fn advisory_severity_reports_findings_without_failing() {
    let (out, report) = check("rule-severity-advisory");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(len(&report, "rules"), 0, "{report}");
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 1, "{report}");
    assert_eq!(advisories[0]["rule"], "no-empty-or-comments-only-files");
    assert_eq!(advisories[0]["file"], "placeholder.ts");
}

#[test]
fn default_and_explicit_error_severity_still_fail() {
    for scenario in ["rule-severity-default", "rule-severity-mixed"] {
        let (out, report) = check(scenario);
        assert_eq!(out.status.code(), Some(1), "{scenario}: {}", stderr(&out));
        assert!(len(&report, "rules") >= 1, "{scenario}: {report}");
    }
}

#[test]
fn mixed_applications_keep_the_rule_failing_without_demoting_it() {
    let (_, report) = check("rule-severity-mixed");
    assert_eq!(len(&report, "advisories"), 0, "{report}");
}

#[test]
fn advisory_severity_moves_codebase_findings_to_advisories() {
    let (out, report) = check("rule-severity-codebase-advisory");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(len(&report, "codebase"), 0, "{report}");
    let advisories = report["advisories"].as_array().unwrap();
    assert!(!advisories.is_empty(), "{report}");
    assert!(advisories.iter().all(|a| a["rule"] == "unique-exports"));
}

#[test]
fn suppressed_advisory_rule_findings_land_only_in_suppressed() {
    // Order-independent: suppression covers both `rules` and `advisories`, so
    // the result is the same whether demotion runs before or after it.
    let (out, report) = check_with("rule-severity-suppressed", &["--include-suppressed"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(len(&report, "rules"), 0, "{report}");
    assert_eq!(len(&report, "advisories"), 0, "{report}");
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{report}");
    assert_eq!(suppressed[0]["rule"], "no-empty-or-comments-only-files");
}
