use std::path::PathBuf;
use std::process::Command;

#[test]
fn checkout_unknown_policy_controls_exit_status_at_max_and_low_fallback() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/github-actions-checkout-credentials/fixture/unanalyzable",
        ),
    );
    for effort in ["max", "low"] {
        for (policy, failures, advice) in [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)] {
            let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!(".no-mistakes.only-{policy}.yml")))
                .args(["--format", "json", "--effort", effort])
                .output()
                .unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.success(), failures == 0, "{stdout}\n{stderr}");
            let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
            assert_eq!(
                report["rules"].as_array().unwrap().len(),
                failures,
                "{report}"
            );
            assert_eq!(
                report["advisories"].as_array().map_or(0, Vec::len),
                advice,
                "{report}"
            );
        }
    }
}

#[test]
fn trusted_target_refs_keep_independent_risks_and_safe_base_checkouts() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-checkout-credentials/fixture/ref-safety"),
    );
    for (name, findings, advisories) in [
        (".no-mistakes.yml", 53, 0),
        (".no-mistakes.max-advise.yml", 26, 27),
        (".no-mistakes.max-ignore.yml", 26, 0),
        (".no-mistakes.configured-report.yml", 51, 0),
        (".no-mistakes.configured-advise.yml", 26, 25),
        (".no-mistakes.configured-ignore.yml", 26, 0),
        (".no-mistakes.safe.yml", 0, 0),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(name))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.success(), findings == 0, "{report}");
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{report}"
        );
    }
}
