use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_lock_reason_check_json_snapshot() {
    let cases = [
        (
            "pass",
            rules_path("postgres-lock-reason", "fixture", "pass"),
            rules_path("postgres-lock-reason", "fixture", "pass/.no-mistakes.yml"),
        ),
        (
            "advisory",
            rules_path("postgres-lock-reason", "fixture", "advisory"),
            rules_path(
                "postgres-lock-reason",
                "fixture",
                "advisory/.no-mistakes.yml",
            ),
        ),
        (
            "error",
            rules_path("postgres-lock-reason", "fixture", "error"),
            rules_path("postgres-lock-reason", "fixture", "error/.no-mistakes.yml"),
        ),
        (
            "unanalyzable-advise",
            rules_path("postgres-lock-reason", "fixture", "unanalyzable"),
            rules_path("postgres-lock-reason", "fixture", "unanalyzable/advise.yml"),
        ),
        (
            "unanalyzable-report",
            rules_path("postgres-lock-reason", "fixture", "unanalyzable"),
            rules_path("postgres-lock-reason", "fixture", "unanalyzable/report.yml"),
        ),
        (
            "unanalyzable-ignore",
            rules_path("postgres-lock-reason", "fixture", "unanalyzable"),
            rules_path("postgres-lock-reason", "fixture", "unanalyzable/ignore.yml"),
        ),
        (
            "suppressed",
            rules_path("postgres-lock-reason", "fixture", "suppressed"),
            rules_path(
                "postgres-lock-reason",
                "fixture",
                "suppressed/.no-mistakes.yml",
            ),
        ),
        (
            "local-origin",
            rules_path("postgres-lock-reason", "fixture", "local-origin"),
            rules_path(
                "postgres-lock-reason",
                "fixture",
                "local-origin/.no-mistakes.yml",
            ),
        ),
        (
            "co-demand",
            rules_path("postgres-lock-reason", "fixture", "co-demand"),
            rules_path(
                "postgres-lock-reason",
                "fixture",
                "co-demand/.no-mistakes.yml",
            ),
        ),
        (
            "co-demand-inventory-only",
            rules_path("postgres-lock-reason", "fixture", "co-demand"),
            rules_path(
                "postgres-lock-reason",
                "fixture",
                "co-demand/inventory-only.yml",
            ),
        ),
    ];
    let reports = assert_saved_fixture_report("postgres_lock_reason", &cases, &[], &[]);
    assert_eq!(reports["pass"]["exitCode"], 0);
    assert_eq!(reports["pass"]["report"]["rules"], serde_json::json!([]));
    assert_eq!(
        reports["pass"]["report"]["advisories"],
        serde_json::json!([])
    );

    assert_eq!(reports["advisory"]["exitCode"], 0);
    assert_eq!(
        reports["advisory"]["report"]["rules"],
        serde_json::json!([])
    );
    assert!(!reports["advisory"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());

    assert_eq!(reports["error"]["exitCode"], 1);
    assert_eq!(
        reports["error"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        5
    );

    assert_eq!(reports["unanalyzable-advise"]["exitCode"], 0);
    assert_eq!(
        reports["unanalyzable-advise"]["report"]["advisories"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(reports["unanalyzable-report"]["exitCode"], 1);
    assert_eq!(
        reports["unanalyzable-report"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(reports["unanalyzable-ignore"]["exitCode"], 0);
    assert_eq!(
        reports["unanalyzable-ignore"]["report"]["rules"],
        serde_json::json!([])
    );
    assert_eq!(
        reports["unanalyzable-ignore"]["report"]["advisories"],
        serde_json::json!([])
    );

    assert_eq!(reports["suppressed"]["exitCode"], 0);
    assert_eq!(
        reports["suppressed"]["report"]["rules"],
        serde_json::json!([])
    );
    assert_eq!(
        reports["suppressed"]["report"]["advisories"],
        serde_json::json!([])
    );

    assert_eq!(reports["local-origin"]["exitCode"], 0);
    assert_eq!(
        reports["local-origin"]["report"]["rules"],
        serde_json::json!([])
    );

    assert_eq!(
        reports["co-demand"]["report"]["rules"],
        reports["co-demand-inventory-only"]["report"]["rules"]
    );
    assert_eq!(
        reports["co-demand"]["report"]["advisories"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn postgres_lock_reason_rejects_invalid_options_at_config_load() {
    let root = rules_path("postgres-lock-reason", "fixture", "invalid-options");
    for (config, expected) in [
        ("unknown-key.yml", "unknown field `unknownKnob`"),
        ("empty-helper.yml", "helpers must be non-empty names"),
        ("invalid-unanalyzable.yml", "invalid options.unanalyzable"),
        ("invalid-glob.yml", "sqlInclude"),
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["--lock-timeout", "0", "check", "--root"])
            .arg(&root)
            .args(["--config", config, "--format", "json"])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(2),
            "{config}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(expected), "{config}: {stderr}");
    }
}
