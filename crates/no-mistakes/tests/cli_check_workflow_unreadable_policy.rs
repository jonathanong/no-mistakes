use std::{path::PathBuf, process::Command};

#[test]
fn unreadable_workflow_policy_keeps_definite_expression_failures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-script-expressions/fixture/unreadable-input");
    for prefix in [
        "",
        "collision-",
        "mixed-",
        "composite-",
        "suppressed-",
        "unrelated-",
    ] {
        for (policy, failures, advice) in [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)] {
            let expected = match prefix {
                "collision-" => (1, 0),
                "mixed-" => (failures + 1, advice),
                "suppressed-" | "unrelated-" => (0, 0),
                _ => (failures, advice),
            };
            let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!("{prefix}{policy}.yml")))
                .args(["--format", "json"])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(i32::from(expected.0 > 0)),
                "{prefix}{policy}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                report["rules"].as_array().unwrap().len(),
                expected.0,
                "{report}"
            );
            assert_eq!(
                report["advisories"].as_array().unwrap().len(),
                expected.1,
                "{report}"
            );
        }
    }
}
