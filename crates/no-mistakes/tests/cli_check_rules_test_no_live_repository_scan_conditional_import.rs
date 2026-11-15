use serde_json::Value;

fn cases() -> Vec<(&'static str, std::path::PathBuf, std::path::PathBuf)> {
    [
        "json-and-css",
        "unknown-offset-external",
        "unknown-offset-local",
    ]
    .into_iter()
    .map(|name| {
        let root = crate::check_json_snapshot::rules_path(
            "test-no-live-repository-scan",
            "conditional-import",
            name,
        );
        (name, root.clone(), root.join(".no-mistakes.yml"))
    })
    .collect()
}

fn advisory_message(report: &Value, name: &str) -> String {
    report[name]["report"]["advisories"][0]["message"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

#[test]
fn conditional_import_check_json_snapshot() {
    let cases = cases();
    let report = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_conditional_import",
        &cases,
        &[],
        &[],
    );
    let named = advisory_message(&report, "json-and-css");
    assert!(named.contains("setup.mts:"), "{named}");
    assert!(named.contains("only inside a conditional"), "{named}");
    let external = advisory_message(&report, "unknown-offset-external");
    assert!(external.contains("only inside a conditional"), "{external}");
    let local = advisory_message(&report, "unknown-offset-local");
    assert!(!local.contains("only inside a conditional"), "{local}");
    assert!(!local.contains("setup.mts:"), "{local}");
}
