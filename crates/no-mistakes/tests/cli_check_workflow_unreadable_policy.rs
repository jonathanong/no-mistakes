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
                super::policy_advisories::without_hygiene(&report).len(),
                expected.1,
                "{report}"
            );
        }
    }
}

#[test]
fn unreadable_workflow_suppressions_remain_auditable_when_requested() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-script-expressions/fixture/unreadable-input");
    for policy in ["report", "advise", "ignore"] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(format!("suppressed-{policy}.yml")))
            .args(["--include-suppressed", "--format", "json"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(report["rules"].as_array().unwrap().is_empty());
        assert!(super::policy_advisories::without_hygiene(&report).is_empty());
        let suppressed = report["suppressed"].as_array().unwrap();
        assert_eq!(
            suppressed.len(),
            if policy == "ignore" { 0 } else { 3 },
            "{report}"
        );
        for row in suppressed {
            assert_eq!(row["rule"], "github-actions-script-expressions");
            assert_eq!(
                row["domain"],
                if policy == "advise" {
                    "advisories"
                } else {
                    "filesystem"
                }
            );
            assert!(row["reason"]
                .as_str()
                .unwrap()
                .contains("cannot read GitHub Actions source"));
        }
        if policy != "ignore" {
            let kinds: Vec<_> = suppressed
                .iter()
                .map(|row| row["directive"]["kind"].as_str().unwrap())
                .collect();
            assert_eq!(kinds, ["file", "line", "nextLine"]);
        }
    }
}
