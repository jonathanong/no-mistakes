use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    config
}

fn fixture_root(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-overrides-ban/fixture")
            .join(scenario),
    )
}

fn findings_for(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture_root(scenario);
    let config_path = root.join(".no-mistakes.yml");
    check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap()
}

#[test]
fn pass_fixture_allows_package_extensions() {
    let findings = findings_for("pass");
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn fail_workspace_overrides() {
    let findings = findings_for("fail-workspace");
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .message
        .contains("top-level \"overrides\" is banned"));
    assert_eq!(findings[0].file, "pnpm-workspace.yaml");
}

#[test]
fn fail_package_json_overrides() {
    let findings = findings_for("fail-package-overrides");
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("packages/app/package.json: top-level \"overrides\" is banned")),
        "{findings:?}"
    );
}

#[test]
fn fail_package_json_pnpm_overrides() {
    let findings = findings_for("fail-pnpm-overrides");
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("packages/web/package.json: \"pnpm.overrides\" is banned")),
        "{findings:?}"
    );
}

#[test]
fn fail_unparseable_workspace_yaml() {
    let findings = findings_for("fail-yaml");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("failed to parse YAML"));
}

#[test]
fn skips_non_mapping_workspace_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("pnpm-workspace.yaml"), "[]\n").unwrap();
    let findings = check(root, &config()).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn skips_non_object_package_json() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("package.json"), "[1]\n").unwrap();
    let files = vec![root.join("package.json")];
    let findings = check_with_files(root, &config(), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn skips_unreadable_workspace_and_invalid_json() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let missing = root.join("pnpm-workspace.yaml");
    let invalid = root.join("package.json");
    std::fs::write(&invalid, "{not json").unwrap();
    let files = vec![missing, invalid];
    let findings = check_with_files(root, &config(), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn check_with_files_and_sources_flags_both_package_override_forms() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let path = root.join("package.json");
    std::fs::write(
        &path,
        r#"{"overrides":{"hono":"1.0.0"},"pnpm":{"overrides":{"fast-uri":"1.0.0"}}}"#,
    )
    .unwrap();
    let files = vec![path];
    let sources = super::super::source_store_for_files(&files);
    let findings = check_with_files_and_sources(root, &config(), &files, &sources).unwrap();
    assert_eq!(findings.len(), 2);
}
