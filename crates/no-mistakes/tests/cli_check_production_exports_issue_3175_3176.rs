use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn production_exports_issues_3175_3176_check_json_snapshot() {
    let mut rows = Vec::new();
    for (case, off_label) in [
        ("issue-3175", "issue_3175_off"),
        ("issue-3175-controls", "issue_3175_controls_off"),
        ("issue-3176", "issue_3176_off"),
        ("issue-3176-suppressed", "issue_3176_suppressed_off"),
    ] {
        let root = rules_path("production-exports-only-used-by-tests", "fixture", case);
        rows.push((case, root.clone(), root.join(".no-mistakes.yml")));
        if case == "issue-3176" {
            rows.push(("issue_3176_error", root.clone(), root.join("error.yml")));
        }
        rows.push((off_label, root.clone(), root.join("off.yml")));
    }
    let uncertainty = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "uncertain",
    );
    for (label, config) in [
        ("discarded_report", ".no-mistakes.yml"),
        ("discarded_advise", "advise.yml"),
        ("discarded_ignore", "ignore.yml"),
    ] {
        rows.push((label, uncertainty.clone(), uncertainty.join(config)));
    }
    assert_saved_fixture("production_exports_issues_3175_3176", &rows);

    let positive = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "issue-3175",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(positive)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["advisories"].as_array().unwrap().is_empty(),
        "{report}"
    );
    let mut targets: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    targets.sort_unstable();
    assert_eq!(targets, vec!["unusedControl"; 6], "{report}");

    for case in ["issue-3175-controls", "issue-3176"] {
        let root = rules_path("production-exports-only-used-by-tests", "fixture", case);
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(root)
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let advisories = report["advisories"].as_array().unwrap();
        assert!(!advisories.is_empty(), "{case}: {report}");
        if case == "issue-3176" {
            assert_eq!(advisories.len(), 4, "{case}: {report}");
        }
        for advisory in advisories {
            let message = advisory.to_string();
            assert!(!message.contains("use a literal"), "{case}: {message}");
            assert!(
                !message.contains("literal named import"),
                "{case}: {message}"
            );
        }
    }

    let runtime = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "issue-3176",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(runtime.clone())
        .args(["--config"])
        .arg(runtime.join("error.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "expected unanalyzable:report to fail"
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 4, "{report}");
    assert!(
        report["advisories"].as_array().unwrap().is_empty(),
        "{report}"
    );

    let suppressed = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "issue-3176-suppressed",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(suppressed)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["advisories"].as_array().unwrap().is_empty(),
        "{report}"
    );
}
