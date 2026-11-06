use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/package-json-required-fields/fixture")
            .join(name),
    )
}

fn config_yaml() -> &'static str {
    r#"
private: true
type: module
license: UNLICENSED
requireScopedName: true
unscopedNameExceptions: [web]
mainWhenFileExists: index.mts
"#
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(config_yaml()).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// Finding cases moved to cli_pkgcfg_snapshot_package_json_required_fields.

#[test]
fn malformed_and_unreadable_manifests_are_reported() {
    let root = fixture("unanalyzable");
    let files = [
        root.join("packages/broken/package.json"),
        root.join("packages/invalid-utf8/package.json"),
        root.join("packages/missing/package.json"),
    ];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(
        findings
            .iter()
            .all(|finding| finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
}

fn allow_audit_config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "requireScopedName: true\nunscopedNameExceptions: ['used', 'used', '@scope/safe']",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn allow_audit_tracks_raw_findings_and_all_overlapping_entries() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/package-json-required-fields/fixture/allow-audit");
    let files = vec![
        root.join("dirty/package.json"),
        root.join("clean/package.json"),
    ];
    let sources = super::super::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &allow_audit_config(), &files, &sources).unwrap();
    let stale: Vec<_> = findings
        .iter()
        .filter(|finding| finding.rule == "unused-allow-entry")
        .collect();
    assert_eq!(stale.len(), 1, "{findings:?}");
    assert_eq!(stale[0].target.as_deref(), Some("@scope/safe"));
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn allow_audit_requires_the_full_application_universe() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/package-json-required-fields/fixture/allow-audit");
    let inventory = vec![
        root.join("dirty/package.json"),
        root.join("clean/package.json"),
    ];
    let sources = super::super::source_store_for_files(&inventory);
    let findings =
        check_with_files_and_sources(&root, &allow_audit_config(), &inventory[1..], &sources)
            .unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule != "unused-allow-entry"),
        "{findings:?}"
    );
}

#[test]
fn allow_audit_is_silent_when_relevant_input_is_unreadable() {
    let root = fixture("allow-audit");
    let files = vec![root.join("missing/package.json")];
    let sources = super::super::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &allow_audit_config(), &files, &sources).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule != "unused-allow-entry"),
        "{findings:?}"
    );
}

#[test]
fn allow_audit_is_silent_when_manifest_parse_fails() {
    let root = fixture("allow-audit");
    let files = vec![root.join("broken/package.json")];
    let sources = super::super::source_store_for_files(&files);
    let findings =
        check_with_files_and_sources(&root, &allow_audit_config(), &files, &sources).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule != "unused-allow-entry"),
        "{findings:?}"
    );
}

#[test]
fn an_allow_entry_named_unanalyzable_is_still_a_hygiene_result() {
    let root = fixture("allow-audit");
    let mut config = allow_audit_config();
    config.rules[0].options = serde_yaml::from_str(
        "requireScopedName: true\nunscopedNameExceptions: [unanalyzable]\nunanalyzable: ignore",
    )
    .unwrap();
    let files = [root.join("clean/package.json")];
    let sources = super::super::source_store_for_files(&files);
    let output =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert_eq!(output.findings.len(), 1, "{:?}", output.findings);
    assert_eq!(output.findings[0].rule, "unused-allow-entry");
    assert_eq!(output.findings[0].target.as_deref(), Some("unanalyzable"));
}
