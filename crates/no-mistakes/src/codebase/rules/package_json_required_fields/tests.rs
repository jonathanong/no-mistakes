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
