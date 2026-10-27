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

#[test]
fn cli_rust_doc_directives_are_invalid_and_never_suppress_findings() {
    for (fixture, expected_violations, expected_invalid) in [
        (
            "rust-doc-disable-directives",
            [("inner.rs", 5), ("outer.rs", 2)],
            [("inner.rs", 1), ("line.rs", 1), ("outer.rs", 1)],
        ),
        (
            "rust-block-doc-disable-directives",
            [("inner.rs", 7), ("outer.rs", 2)],
            [("decorated.rs", 2), ("inner.rs", 2), ("outer.rs", 1)],
        ),
    ] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check")
            .join(fixture);
        for (config, advice_count, invalid_count) in [
            (".no-mistakes.yml", 3, 0),
            ("report.yml", 0, 3),
            ("ignore.yml", 0, 0),
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(config))
                .args(["--format", "json", "--include-suppressed"])
                .output()
                .unwrap();
            // Ignoring hygiene never hides the real violations under doc comments.
            assert_eq!(output.status.code(), Some(1), "{config}: {output:?}");
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(report["advisories"].as_array().unwrap().len(), advice_count);
            let rules = report["rules"].as_array().unwrap();
            let violations: Vec<_> = rules
                .iter()
                .filter(|row| row["rule"] == "rust-no-inline-tests")
                .map(|row| (row["file"].as_str().unwrap(), row["line"].as_u64().unwrap()))
                .collect();
            assert_eq!(violations, expected_violations, "{fixture}: {report}");
            assert_eq!(rules.len(), 2 + invalid_count, "{report}");
            let invalid: Vec<_> = rules
                .iter()
                .chain(report["advisories"].as_array().unwrap())
                .filter(|row| row["rule"] == "invalid-disable-directive")
                .map(|row| {
                    assert_eq!(row["target"], "rust-no-inline-tests", "{report}");
                    let message = row["message"].as_str().unwrap();
                    assert!(
                        message.contains("plain")
                            && message.contains("//")
                            && message.contains("rust-no-inline-tests"),
                        "{report}"
                    );
                    (row["file"].as_str().unwrap(), row["line"].as_u64().unwrap())
                })
                .collect();
            if config != "ignore.yml" {
                assert_eq!(invalid, expected_invalid, "{fixture}: {report}");
            }
            let suppressed = report["suppressed"].as_array().unwrap();
            assert_eq!(suppressed.len(), 1, "{report}");
            assert_eq!(suppressed[0]["file"], "plain.rs");
            assert_eq!(
                suppressed[0]["directive"],
                serde_json::json!({"kind": "nextLine", "line": 1})
            );
        }
    }
}

#[test]
fn check_json_snapshot_rust_doc_suppressions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/rust-doc-disable-directives");
    let cases = [
        ("advise", ".no-mistakes.yml"),
        ("report", "report.yml"),
        ("ignore", "ignore.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), root.join(config)));
    crate::check_json_snapshot::assert_saved_fixture("rust_doc_suppressions", &cases);
}

#[test]
fn check_json_snapshot_rust_block_doc_suppressions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/rust-block-doc-disable-directives");
    let cases = [
        ("advise", ".no-mistakes.yml"),
        ("report", "report.yml"),
        ("ignore", "ignore.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), root.join(config)));
    crate::check_json_snapshot::assert_saved_fixture("rust_block_doc_suppressions", &cases);
}
