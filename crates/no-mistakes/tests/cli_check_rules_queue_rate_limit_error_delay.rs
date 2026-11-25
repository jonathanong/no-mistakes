use std::path::PathBuf;
use std::process::Command;

fn fixture_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/queue-rate-limit-error-delay")
}

#[test]
fn queue_rate_limit_error_delay_check_json_snapshot() {
    let base = fixture_base();
    let root = base.join("fixture");
    let configs = base.join("configs");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_rate_limit_error_delay",
        &[
            ("default-off", root.clone(), configs.join("default-off.yml")),
            ("advise", root.clone(), configs.join("advise.yml")),
            ("report", root.clone(), configs.join("report.yml")),
            ("ignore", root.clone(), configs.join("ignore.yml")),
        ],
        &[],
        &[],
    );

    assert_eq!(reports["default-off"]["exitCode"], 0);
    assert_eq!(
        reports["default-off"]["report"]["rules"],
        serde_json::json!([])
    );
    assert_eq!(
        reports["default-off"]["report"]["advisories"],
        serde_json::json!([])
    );

    assert_eq!(reports["advise"]["exitCode"], 1);
    let advise_rules = reports["advise"]["report"]["rules"].as_array().unwrap();
    let advise_advisories = reports["advise"]["report"]["advisories"]
        .as_array()
        .unwrap();
    assert!(!advise_rules.is_empty());
    assert!(!advise_advisories.is_empty());
    assert!(advise_rules.iter().all(|row| {
        let file = row["file"].as_str().unwrap_or("");
        file == "src/violations.ts"
    }));
    assert!(advise_advisories
        .iter()
        .all(|row| { row["file"] == "src/unanalyzable.ts" && row["target"] == "unanalyzable" }));

    assert_eq!(reports["report"]["exitCode"], 1);
    assert_eq!(
        reports["report"]["report"]["advisories"],
        serde_json::json!([])
    );
    let report_rules = reports["report"]["report"]["rules"].as_array().unwrap();
    assert_eq!(
        report_rules.len(),
        advise_rules.len() + advise_advisories.len()
    );

    assert_eq!(reports["ignore"]["exitCode"], 1);
    assert_eq!(
        reports["ignore"]["report"]["advisories"],
        serde_json::json!([])
    );
    assert_eq!(
        reports["ignore"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        advise_rules.len()
    );
}

#[test]
fn queue_rate_limit_error_delay_invalid_config_exits_2() {
    let base = fixture_base();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "--lock-timeout",
            "0",
            "check",
            "--root",
            base.join("fixture").to_str().unwrap(),
            "--config",
            base.join("configs/invalid-empty.yml").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(2),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("queue-rate-limit-error-delay") && stderr.contains("options.targets"),
        "{stderr}"
    );
}
