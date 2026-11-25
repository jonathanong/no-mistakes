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
        "wrapped-fixtures",
        "wrapped-fixtures-configured",
    ]
    .map(|name| {
        let root = match name {
            "workflow-action" | "blank-reason" => {
                rules_path("test-no-retries", "fixture", "workflow-action")
            }
            "malformed" | "static-shapes" | "playwright-export-shapes" | "followups" => {
                rules_path("test-no-retries", "fixture", name)
            }
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
