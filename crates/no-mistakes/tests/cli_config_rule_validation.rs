use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use no_mistakes::codebase::rules::rule_ids::configurable_rule_ids;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn invalid_rule_configuration_aborts_before_analysis() {
    let root = rules_path("options-audit", "fixture", "issue-4110");
    let cases = [
        ("unknown", root.clone(), root.join("unknown.yml")),
        ("unknown-map", root.clone(), root.join("unknown-map.yml")),
        (
            "unknown-override",
            root.clone(),
            root.join("unknown-override.yml"),
        ),
        (
            "unknown-project",
            root.clone(),
            root.join("unknown-project.yml"),
        ),
        ("unknown-jsonc", root.clone(), root.join("unknown.jsonc")),
        (
            "invalid-options",
            root.clone(),
            root.join("invalid-options.yml"),
        ),
        (
            "invalid-values",
            root.clone(),
            root.join("invalid-values.yml"),
        ),
        ("fragment", root.join("fragment"), PathBuf::new()),
    ];
    let result = assert_saved_fixture_report("invalid_rule_configuration", &cases, &[], &[]);
    for (label, _, _) in cases {
        assert_eq!(result[label]["exitCode"], 2, "{label}: {result}");
        assert!(result[label]["report"].is_null(), "{label}: {result}");
    }
}

#[test]
fn every_registered_rule_rejects_an_unknown_option() {
    let root = rules_path("options-audit", "fixture", "issue-4110");
    let path = root.join("all-rules.yml");
    let yaml: Value = serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let rules = yaml["rules"].as_sequence().unwrap();
    let declared: BTreeSet<_> = rules
        .iter()
        .map(|rule| {
            assert_eq!(rule["options"]["bogusOption"].as_bool(), Some(true));
            rule["rule"].as_str().unwrap()
        })
        .collect();
    let registered: BTreeSet<_> = configurable_rule_ids().collect();
    assert!(!registered.is_empty());
    assert_eq!(
        declared, registered,
        "update the saved fixture when registering a rule"
    );
    assert_eq!(rules.len(), registered.len());

    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "{stderr}");
    for id in registered {
        let diagnostic = stderr
            .lines()
            .find(|line| line.contains(&format!("`{id}`")));
        assert!(
            diagnostic.is_some_and(|line| line.contains("bogusOption") && line.contains("all-rules.yml:")),
            "missing bogusOption diagnostic for {id}: {stderr}"
        );
    }
}
