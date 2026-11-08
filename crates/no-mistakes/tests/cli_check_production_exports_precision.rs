use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::process::Command;

const CASES: &[&str] = &[
    "overloads-tests",
    "overloads-production",
    "ambient",
    "augmentation",
    "runtime-controls",
    "computed",
    "clean",
];

#[test]
fn production_exports_precision_check_json_snapshot() {
    assert_saved_fixture("production_exports_precision", &cases(".no-mistakes.yml"));
}

#[test]
fn production_exports_precision_rule_off_check_json_snapshot() {
    assert_saved_fixture("production_exports_precision_rule_off", &cases(".off.yml"));
}

fn cases(config: &str) -> Vec<(&'static str, std::path::PathBuf, std::path::PathBuf)> {
    CASES
        .iter()
        .map(|case| {
            let root = rules_path(
                "production-exports-only-used-by-tests",
                "fixture",
                &format!("precision/{case}"),
            );
            (*case, root.clone(), root.join(config))
        })
        .collect()
}

#[test]
fn precision_controls_preserve_enforcement_and_site_counts() {
    for (case, violations, advisories) in [
        ("overloads-tests", 1, 0),
        ("overloads-production", 0, 0),
        ("ambient", 0, 0),
        ("augmentation", 0, 0),
        ("runtime-controls", 1, 4),
        ("computed", 0, 3),
        ("clean", 0, 0),
    ] {
        let root = rules_path(
            "production-exports-only-used-by-tests",
            "fixture",
            &format!("precision/{case}"),
        );
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(".no-mistakes.yml"))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            violations,
            "{case}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{case}: {report}"
        );
        assert_eq!(
            output.status.code(),
            Some(i32::from(violations > 0)),
            "{case}"
        );
    }
}
