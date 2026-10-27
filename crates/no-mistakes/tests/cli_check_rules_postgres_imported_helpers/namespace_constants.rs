use super::{unanalyzable, Findings, UNANALYZABLE};
use std::path::PathBuf;
use std::process::Command;

fn check(effort: &str) -> (serde_json::Value, Findings) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-required-predicates/variants/namespace-const-sql");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--effort", effort])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
    assert_eq!(output.status.code(), Some(1), "{effort}: {report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
    let mut findings: Findings = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap().to_string(),
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    findings.sort();
    (report, findings)
}

#[test]
fn namespace_constants_keep_precise_max_findings() {
    let (report, findings) = check("max");
    let mut expected = unanalyzable(&[16, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 31, 35]);
    // Whole-query constants retain the declaration's location for the missing predicate.
    expected.push(("src/constants.mts".to_string(), 5, "orders".to_string()));
    expected.sort();
    assert_eq!(findings, expected, "{report}");
}

#[test]
fn namespace_constants_outside_low_grammar_report_the_static_rewrite() {
    let (report, findings) = check("low");
    // Low does not perform max's cross-module namespace-constant splicing.
    // Previously this test requested low but silently exercised the max fallback.
    assert_eq!(
        findings,
        unanalyzable(&[
            9, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 31, 33, 34,
            35,
        ]),
        "{report}"
    );
    for finding in report["rules"].as_array().unwrap() {
        assert_eq!(finding["target"], UNANALYZABLE);
        let message = finding["message"].as_str().unwrap();
        for text in ["effort low", "literal string", "one local const"] {
            assert!(message.contains(text), "{finding}");
        }
    }
}
