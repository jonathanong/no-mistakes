use std::path::PathBuf;
use std::process::{Command, Output};

const RULE: &str = "production-exports-only-used-by-tests";

fn check(scenario: &str) -> Output {
    check_config(scenario, ".no-mistakes.yml")
}

fn check_config(scenario: &str, config: &str) -> Output {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-exports-only-used-by-tests/fixture")
            .join(scenario),
    );
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn computed_imports_preserve_definite_findings_and_failure_status_for_every_policy() {
    for policy in ["report", "advise", "ignore"] {
        let output = check_config("policy-isolation", &format!("{policy}.yml"));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1), "{policy}: {report}");
        let rules = report["rules"].as_array().unwrap();
        let definite: Vec<_> = rules
            .iter()
            .filter(|row| row["target"] != "unanalyzable")
            .collect();
        assert_eq!(definite.len(), 1, "{policy}: {report}");
        assert_eq!(definite[0]["target"], "testOnly");
        assert_eq!(
            rules.iter().any(|row| row["target"] == "unanalyzable"),
            policy == "report"
        );
        assert_eq!(
            !report["advisories"].as_array().unwrap().is_empty(),
            policy == "advise"
        );
    }
}

#[test]
fn production_exports_test_only_reports_symbol_consumers_through_barrels() {
    let output = check("fail");
    let body = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{body}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    for symbol in [
        "createLinkPost",
        "getRequestClientInfo",
        "createFlowProducer",
        "onlyTest",
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| { finding["rule"] == RULE && finding.to_string().contains(symbol) }),
            "missing {symbol}: {body}"
        );
    }
    // Production consumption belongs to the selected symbol, not its entire file.
    assert!(!findings.iter().any(|finding| {
        finding["rule"] == RULE && finding.to_string().contains("realProduction")
    }));
}

#[test]
fn production_exports_test_only_ignores_production_consumers_and_suppressions() {
    for scenario in ["pass", "suppressed"] {
        let output = check(scenario);
        let body = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{scenario}: {body}");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(report["rules"].as_array().unwrap().is_empty(), "{body}");
    }
}
