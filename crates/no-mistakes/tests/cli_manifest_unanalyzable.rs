use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(rule: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("fixture/unanalyzable")
}

// Both CLI channels and suppression auditing use the prepared source session.
#[test]
fn manifest_failures_follow_policy_without_hiding_definite_findings() {
    for rule in [
        "package-json-registry-only",
        "pnpm-overrides-ban",
        "pnpm-release-age-policy",
    ] {
        let root = fixture(rule);
        for effort in ["max", "low"] {
            for (policy, failures, advice) in [("report", 3, 0), ("advise", 1, 2), ("ignore", 1, 0)]
            {
                for custom in ["", "custom-"] {
                    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                        .args(["check", "--root"])
                        .arg(&root)
                        .arg("--config")
                        .arg(root.join(format!("{custom}{effort}-{policy}.yml")))
                        .args(["--format", "json", "--include-suppressed"])
                        .output()
                        .unwrap();
                    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
                        .unwrap_or_else(|error| {
                            panic!(
                                "{rule}: {error}: {}",
                                String::from_utf8_lossy(&output.stderr)
                            )
                        });
                    let findings = report["rules"].as_array().unwrap();
                    let advisories = report["advisories"].as_array().unwrap();
                    assert_eq!(
                        findings.len(),
                        failures,
                        "{rule} {effort} {policy}: {report}"
                    );
                    assert_eq!(
                        advisories.len(),
                        advice,
                        "{rule} {effort} {policy}: {report}"
                    );
                    assert_eq!(output.status.code(), Some(1));
                    assert_eq!(
                        findings
                            .iter()
                            .filter(|finding| finding["target"] != "unanalyzable")
                            .count(),
                        1
                    );
                    if policy != "ignore" {
                        assert!(
                            report["suppressed"]
                                .to_string()
                                .contains("packages/suppressed/package.json"),
                            "{report}"
                        );
                        assert!(
                            report["suppressed"]
                                .to_string()
                                .contains("packages/suppressed-utf8/package.json"),
                            "{report}"
                        );
                    }
                    for finding in findings.iter().chain(advisories) {
                        assert!(
                            !finding["file"].as_str().unwrap().contains("suppressed"),
                            "{report}"
                        );
                        if !custom.is_empty() {
                            assert!(
                                finding["message"]
                                    .as_str()
                                    .unwrap()
                                    .starts_with("Repair dependency policy metadata."),
                                "{report}"
                            );
                        } else if finding["target"] == "unanalyzable" {
                            assert!(
                                finding["message"]
                                    .as_str()
                                    .unwrap()
                                    .contains("could not be read or parsed"),
                                "{report}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn bom_manifests_remain_analyzable_without_hiding_definite_violations() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/manifest-analysis/fixture/utf8-bom");
    for (scenario, expected) in [("valid", 0), ("violation", 3)] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(fixture.join(scenario))
            .arg("--config")
            .arg(fixture.join("check.yml"))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(output.status.success(), expected == 0, "{report}");
        assert_eq!(findings.len(), expected, "{report}");
        if scenario == "violation" {
            let mut rules = findings
                .iter()
                .map(|finding| finding["rule"].as_str().unwrap())
                .collect::<Vec<_>>();
            rules.sort_unstable();
            assert_eq!(
                rules,
                [
                    "package-json-registry-only",
                    "package-json-required-fields",
                    "pnpm-overrides-ban"
                ]
            );
        }
        assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
        assert!(
            findings
                .iter()
                .all(|finding| finding["target"] != "unanalyzable"),
            "{report}"
        );
    }
}
