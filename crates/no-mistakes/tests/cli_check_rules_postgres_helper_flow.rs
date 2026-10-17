//! Executor SQL recovered through builder `.text` and helper control flow.
//! Every scenario is a saved fixture under `test-cases/rules/<rule>/variants/`.

use std::path::PathBuf;
use std::process::Command;

const REQUIRED: &str = "postgres-required-predicates";
const VIEW: &str = "view_active_plans";
const UNANALYZABLE: &str = "unanalyzable";

/// `(file, line, target)` of every finding, sorted.
type Findings = Vec<(String, u64, String)>;

fn required(name: &str) -> Findings {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(REQUIRED)
        .join("variants")
        .join(name);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
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
    findings
}

fn expect(rows: &[(&str, u64, &str)]) -> Findings {
    let mut rows: Findings = rows
        .iter()
        .map(|(file, line, target)| (file.to_string(), *line, target.to_string()))
        .collect();
    rows.sort();
    rows
}

#[test]
fn builder_text_executor_arguments_are_recovered_and_fail_closed() {
    // Lines 6 and 13 are the builders' own SQL; every later case must stay
    // unanalyzable (unknown builder, extra interpolation, foreign `.values`,
    // plain object, partially recovered builder).
    assert_eq!(
        required("builder-text"),
        expect(&[
            ("src/caller.mts", 6, VIEW),
            ("src/caller.mts", 13, VIEW),
            ("src/caller.mts", 22, UNANALYZABLE),
            ("src/caller.mts", 28, UNANALYZABLE),
            ("src/caller.mts", 35, UNANALYZABLE),
            ("src/caller.mts", 41, UNANALYZABLE),
            ("src/caller.mts", 48, UNANALYZABLE),
        ])
    );
}
