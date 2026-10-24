use std::{path::PathBuf, process::Command};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/file-input-analysis/fixture")
}

#[test]
fn file_input_analysis_routes_every_rule_through_policy_at_max_and_low_fallback() {
    let root = fixture();
    for effort in ["max", "low"] {
        for prefix in ["mixed-", "only-"] {
            for (policy, uncertainty, advisories) in
                [("report", 12, 0), ("advise", 0, 12), ("ignore", 0, 0)]
            {
                let definite = usize::from(prefix == "mixed-") * 9;
                let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                    .args(["check", "--root"])
                    .arg(&root)
                    .arg("--config")
                    .arg(root.join(format!("{prefix}{policy}.yml")))
                    .args(["--effort", effort, "--format", "json"])
                    .output()
                    .unwrap();
                let body = String::from_utf8_lossy(&output.stdout);
                let report: serde_json::Value = serde_json::from_slice(&output.stdout)
                    .unwrap_or_else(|error| {
                        panic!(
                            "{prefix}{policy}/{effort}: {error}: {body} {}",
                            String::from_utf8_lossy(&output.stderr)
                        )
                    });
                assert_eq!(
                    output.status.code(),
                    Some(i32::from(definite + uncertainty > 0)),
                    "{report}"
                );
                assert_eq!(
                    report["rules"].as_array().unwrap().len(),
                    definite + uncertainty,
                    "{prefix}{policy}/{effort}: {report}"
                );
                assert_eq!(
                    report["advisories"].as_array().unwrap().len(),
                    advisories,
                    "{prefix}{policy}/{effort}: {report}"
                );
                assert!(!body.contains("suppressed."), "{report}");
            }
        }
    }
}

#[test]
fn file_input_analysis_retained_byte_directives_remain_auditable() {
    let root = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("mixed-report.yml"))
        .args(["--include-suppressed", "--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 21, "{report}");
    let suppressed = report["suppressed"].to_string();
    assert!(suppressed.contains("suppressed."), "{report}");
    assert_eq!(output.status.code(), Some(1));
}
