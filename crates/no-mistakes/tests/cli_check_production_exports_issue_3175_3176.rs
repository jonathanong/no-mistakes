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

    let mocks = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "folded-mock-imports",
    );
    let mock_rows: Vec<_> = [
        ("advise", ".no-mistakes.yml"),
        ("report", "report.yml"),
        ("ignore", "ignore.yml"),
        ("off", "off.yml"),
    ]
    .into_iter()
    .map(|(label, config)| (label, mocks.clone(), mocks.join(config)))
    .collect();
    assert_saved_fixture("production_exports_folded_mocks", &mock_rows);
    let depth = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "static-dynamic-facts/depth-cache",
    );
    assert_saved_fixture(
        "production_exports_folded_depth",
        &[
            ("advise", depth.clone(), depth.join(".no-mistakes.yml")),
            ("off", depth.clone(), depth.join("off.yml")),
        ],
    );
    for (_, root, config) in mock_rows.iter().filter(|(label, _, _)| *label != "off") {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(root)
            .arg("--config")
            .arg(config)
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(
            report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|finding| finding["target"] == "isolated"),
            "{report}"
        );
    }

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
            assert_runtime_import_next_step(&advisory.to_string());
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
    let rules = report["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 4, "{report}");
    for finding in rules {
        assert_runtime_import_next_step(&finding.to_string());
    }
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

fn assert_runtime_import_next_step(message: &str) {
    assert!(!message.contains("use a literal"), "{message}");
    assert!(!message.contains("literal named import"), "{message}");
    if !message.contains("Runtime-resolved import") {
        return;
    }
    assert!(
        message.contains(concat!(
            "// no-mistakes-disable-line production-exports-only-used-by-tests",
            " -- runtime-resolved import boundary"
        )),
        "{message}"
    );
    assert!(!message.contains("use explicit named imports"), "{message}");
}
