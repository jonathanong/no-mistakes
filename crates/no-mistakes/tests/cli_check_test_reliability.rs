use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::process::Command;

#[test]
fn test_reliability_check_json_snapshot() {
    let root = rules_path("test-no-retries", "fixture", "report");
    let cases = [
        "default",
        "allowlist",
        "workflow-action",
        "blank-reason",
        "malformed",
        "static-shapes",
        "followups",
        "local-playwright",
        "local-playwright-default",
        "playwright-export-shapes",
        "serial-disable-line",
        "wrapped-fixtures",
        "wrapped-fixtures-configured",
    ]
    .map(|name| {
        let root = match name {
            "workflow-action" | "blank-reason" => {
                rules_path("test-no-retries", "fixture", "workflow-action")
            }
            "malformed"
            | "static-shapes"
            | "playwright-export-shapes"
            | "followups"
            | "serial-disable-line" => rules_path("test-no-retries", "fixture", name),
            "local-playwright" | "local-playwright-default" => {
                rules_path("test-no-retries", "fixture", "local-playwright")
            }
            "wrapped-fixtures" | "wrapped-fixtures-configured" => {
                rules_path("test-no-retries", "fixture", "wrapped-fixtures")
            }
            _ => root.clone(),
        };
        (
            name,
            root,
            rules_path("test-no-retries", "configs", &format!("{name}.yml")),
        )
    });
    assert_saved_fixture("test_reliability", &cases);
}

#[test]
fn test_no_retries_line_suppression_json_matches_snapshot() {
    assert_saved_fixture(
        "test_no_retries_line_suppression",
        &[(
            "retries-disable-line",
            rules_path("test-no-retries", "fixture", "retries-disable-line"),
            rules_path("test-no-retries", "configs", "retries-disable-line.yml"),
        )],
    );
}

#[test]
fn test_reliability_invalid_options_check_json_snapshot() {
    let root = rules_path("test-no-retries", "fixture", "workflow-action");
    let cases = [
        "absolute-path",
        "parent-path",
        "invalid-glob",
        "both-selectors",
        "invalid-command",
        "negative-retry-cap",
        "unknown-serial-option",
        "invalid-fixture-export",
    ]
    .map(|name| {
        (
            name,
            root.clone(),
            rules_path("test-no-retries", "configs/invalid", &format!("{name}.yml")),
        )
    });
    assert_saved_fixture("test_reliability_invalid_options", &cases);
}

#[test]
fn test_reliability_rejects_invalid_scope_globs() {
    let root = rules_path("test-no-retries", "fixture", "workflow-action");
    for (name, diagnostic) in [
        ("invalid-rule-glob", "tests/[browser"),
        ("invalid-runner-glob", "tests/[browser"),
        (
            "blank-factory-glob",
            "options.files entries must be nonempty globs",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["--lock-timeout", "0", "check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(rules_path(
                "test-no-retries",
                "configs/invalid",
                &format!("{name}.yml"),
            ))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{name}: {error}");
        assert!(error.contains(diagnostic), "{name}: {error}");
    }
}

#[test]
fn uncertain_playwright_factory_check_json_snapshot() {
    let root = rules_path("test-no-retries", "fixture", "uncertain-factory");
    let reports = super::check_json_snapshot::assert_saved_fixture_report(
        "uncertain_playwright_factory",
        &[("uncertain", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["uncertain"]["report"];
    assert_eq!(reports["uncertain"]["exitCode"], 1);
    assert!(report["rules"].as_array().unwrap().is_empty());
    let incomplete = report["incomplete"].as_array().unwrap();
    assert_eq!(incomplete.len(), 2);
    assert!(incomplete.iter().all(|entry| {
        entry["consumer"] == "test-no-retries"
            && entry["message"]
                .as_str()
                .unwrap()
                .contains("local factory return could not be traced statically")
    }));
}

#[test]
fn uncertain_star_playwright_factory_check_json_snapshot() {
    let root = rules_path("test-no-retries", "fixture", "uncertain-star-factory");
    let reports = super::check_json_snapshot::assert_saved_fixture_report(
        "uncertain_star_playwright_factory",
        &[
            ("uncertain", root.clone(), root.join(".no-mistakes.yml")),
            ("configured", root.clone(), root.join("configured.yml")),
        ],
        &[],
        &[],
    );
    let report = &reports["uncertain"]["report"];
    assert_eq!(reports["uncertain"]["exitCode"], 1);
    assert!(report["rules"].as_array().unwrap().is_empty());
    let incomplete = report["incomplete"].as_array().unwrap();
    assert_eq!(incomplete.len(), 1);
    assert!(incomplete[0]["message"]
        .as_str()
        .unwrap()
        .contains("expression in fixture chain could not be traced statically"));
    assert_eq!(reports["configured"]["exitCode"], 0);
    assert!(reports["configured"]["report"]["incomplete"].is_null());
}

#[test]
fn uncertain_playwright_receiver_check_json_snapshot() {
    let root = rules_path("test-no-retries", "fixture", "review-uncertain-binding");
    let reports = super::check_json_snapshot::assert_saved_fixture_report(
        "uncertain_playwright_receiver",
        &[("configured", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["configured"]["report"];
    assert_eq!(reports["configured"]["exitCode"], 1);
    let rules = report["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 4);
    assert!(rules.iter().all(|entry| {
        entry["file"] == "tests/local.spec.mts" && matches!(entry["line"].as_u64(), Some(10 | 13))
    }));
    let incomplete = report["incomplete"].as_array().unwrap();
    assert_eq!(incomplete.len(), 5);
}
