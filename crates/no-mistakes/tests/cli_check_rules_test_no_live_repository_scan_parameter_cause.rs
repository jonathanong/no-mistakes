use std::path::PathBuf;

const RULE: &str = "test-no-live-repository-scan";
const CASES: [&str; 18] = [
    "live-path",
    "omitted-argument",
    "unproven-variable",
    "forwarded-parameter",
    "namespace-import",
    "default-import",
    "dynamic-import",
    "require-import",
    "vi-mock",
    "non-direct-reference",
    "not-named-function",
    "reassigned-parameter",
    "multibyte-argument",
    "multiline-property",
    "anonymous-callback",
    "forwarded-anonymous",
    "temp-climbs",
    "climbing-argument",
];

fn root(name: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "parameter-cause", name)
}

#[test]
fn parameter_cause_check_json_snapshot() {
    let cases: Vec<_> = CASES
        .into_iter()
        .map(|name| {
            let root = root(name);
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_parameter_cause",
        &cases,
    );
}

#[test]
fn multiline_arguments_name_the_selected_value_and_its_line() {
    let root = root("multiline-property");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 3, "{report}");
    for (finding, expected) in advisories.iter().zip([
        "root.mts:20 passes `root`, an unproven variable, as property `root`",
        "root.mts:24 passes `directory`, an unproven variable, as property `directory`",
        "root.mts:27 passes `directory`, an unproven variable",
    ]) {
        assert!(
            finding["message"].as_str().unwrap().contains(expected),
            "{finding}"
        );
    }
}

#[test]
fn temporary_directory_parent_stays_unproven() {
    let root = root("temp-climbs");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let advisories = report["advisories"].as_array().unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    assert_eq!(advisories.len(), 1, "{report}");
    assert_eq!(advisories[0]["file"], "root.mts");
    assert_eq!(advisories[0]["line"], 4);
    assert_eq!(advisories[0]["target"], "unresolved root");
}
