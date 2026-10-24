use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/package-json-required-fields/fixture")
            .join(scenario),
    )
}

fn check_fixture_config(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const RULE: &str = "package-json-required-fields";

#[test]
fn package_json_required_fields_fails_shape() {
    let root = fixture("fail");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("scoped"), "{body}");
    assert!(body.contains("private"), "{body}");
}

#[test]
fn package_json_required_fields_passes() {
    let root = fixture("pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn package_json_required_fields_allows_unscoped_exception() {
    let root = fixture("exception");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn package_json_required_fields_json_has_rule_id() {
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
fn package_json_required_fields_filesystem_runner_discovers_files() {
    let root = fixture("fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(
        findings.iter().any(|finding| {
            finding.rule == no_mistakes::codebase::rules::PACKAGE_JSON_REQUIRED_FIELDS
        }),
        "{body}"
    );
}

// A broken sibling must remain visible without hiding definite field violations.
#[test]
fn malformed_manifest_policy_preserves_definite_findings_and_suppression() {
    let root = fixture("unanalyzable");
    for prefix in ["", "only-"] {
        let ordinary = usize::from(prefix.is_empty());
        for (policy, uncertain, advisories) in
            [("report", 2, 0), ("advise", 0, 2), ("ignore", 0, 0)]
        {
            let out = Command::new(bin())
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!("{prefix}{policy}.yml")))
                .args(["--format", "json"])
                .output()
                .unwrap();
            let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
            let findings = report["rules"].as_array().unwrap();
            let advice = report["advisories"].as_array().unwrap();
            assert_eq!(findings.len(), uncertain + ordinary, "{policy}: {report}");
            assert_eq!(advice.len(), advisories, "{policy}: {report}");
            let has_definite = findings.iter().any(|finding| {
                finding["file"] == "packages/valid/package.json" && finding["target"] == "private"
            });
            assert_eq!(has_definite, ordinary > 0, "{report}");
            for finding in findings.iter().chain(advice) {
                assert_ne!(
                    finding["file"], "packages/suppressed/package.json",
                    "{report}"
                );
                if finding["target"] == "unanalyzable" {
                    assert!(
                        [
                            "packages/broken/package.json",
                            "packages/invalid-utf8/package.json"
                        ]
                        .iter()
                        .any(|file| finding["file"] == *file),
                        "{report}"
                    );
                }
            }
            assert_eq!(out.status.code(), Some(i32::from(uncertain + ordinary > 0)));
        }
    }
}

#[test]
fn malformed_manifest_suppression_is_accounted_by_audit() {
    let root = fixture("unanalyzable");
    let output = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("report.yml"))
        .args(["--include-suppressed", "--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 3, "{report}");
    assert!(
        report["suppressed"]
            .to_string()
            .contains("packages/suppressed/package.json"),
        "{report}"
    );
    assert_eq!(output.status.code(), Some(1));
}
