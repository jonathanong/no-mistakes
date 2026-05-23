use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture_root(subpath: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/doc-consistency")
            .join(subpath),
    )
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture_root("pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}

#[test]
fn fail_fixture_has_findings() {
    let root = fixture_root("fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        !findings.is_empty(),
        "expected findings for missing heading"
    );
}

#[test]
fn missing_required_file_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let config = config_with_rule("requiredFiles: [NONEXISTENT.md]");
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("not found"));
}

#[test]
fn required_heading_present_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("GUIDE.md"), "# Title\n## Related issues\n").unwrap();
    let config =
        config_with_rule("requiredFiles: [GUIDE.md]\nrequiredHeading: \"## Related issues\"");
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn required_heading_missing_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("GUIDE.md"), "# Title\n## Overview\n").unwrap();
    let config =
        config_with_rule("requiredFiles: [GUIDE.md]\nrequiredHeading: \"## Related issues\"");
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing required heading"));
}

#[test]
fn required_substring_present_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("README.md"), "# Title\nSee CONTRIBUTING.md\n").unwrap();
    let config = config_with_rule(
        "requiredFiles: [README.md]\nrequiredSubstrings:\n  - file: README.md\n    substring: CONTRIBUTING.md",
    );
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn required_substring_missing_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("README.md"), "# Title\nNo contributing info.\n").unwrap();
    let config = config_with_rule(
        "requiredFiles: [README.md]\nrequiredSubstrings:\n  - file: README.md\n    substring: CONTRIBUTING.md",
    );
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing required substring"));
}

#[test]
fn banned_substring_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("notes.md"), "# Notes\nDo not use legacy API.\n").unwrap();
    let config = config_with_rule("bannedSubstrings: [\"legacy API\"]");
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("banned substring"));
}

#[test]
fn banned_substring_absent_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("notes.md"), "# Notes\nUse the new API.\n").unwrap();
    let config = config_with_rule("bannedSubstrings: [\"legacy API\"]");
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_works() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let guide = root.join("GUIDE.md");
    std::fs::write(&guide, "# Guide\n").unwrap();
    let config =
        config_with_rule("requiredFiles: [GUIDE.md]\nrequiredHeading: \"## Related issues\"");
    let findings = check_with_files(root, &config, &[guide]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn empty_options_has_no_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("README.md"), "anything\n").unwrap();
    let config = config_with_rule("{}");
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty());
}
