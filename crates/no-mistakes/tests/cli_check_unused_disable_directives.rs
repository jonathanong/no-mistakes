use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn cli_audit_ignores_examples_and_excluded_data_but_keeps_real_comments() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-comment-boundaries");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(
        report["advisories"].as_array().unwrap().len(),
        2,
        "{report}"
    );
    let locations: Vec<_> = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            assert_eq!(row["rule"], "unused-disable-directive", "{report}");
            (row["file"].as_str().unwrap(), row["line"].as_u64().unwrap())
        })
        .collect();
    assert_eq!(locations, [("README.md", 9), ("source.rs", 1)]);
}

#[test]
fn cli_hygiene_modes_preserve_suppression_and_choose_exit_status() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/unused-disable-directives");
    for (config, code, advisory_count, rule_count) in [
        (".no-mistakes.yml", 0, 6, 0),
        ("report.yml", 1, 0, 6),
        ("ignore.yml", 0, 0, 0),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config))
            .args(["--format", "json", "--include-suppressed"])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisory_count,
            "{report}"
        );
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            rule_count,
            "{report}"
        );
        assert_eq!(
            report["suppressed"].as_array().unwrap().len(),
            3,
            "{report}"
        );
    }
}

#[test]
fn cli_react_consumed_comments_do_not_fail_strict_hygiene_or_expand_public_ledger() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/react-consumed-directives/all");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("report.yml"))
        .args(["--format", "json", "--include-suppressed"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
    assert_eq!(
        report["suppressed"].as_array().unwrap().len(),
        1,
        "{report}"
    );
}
