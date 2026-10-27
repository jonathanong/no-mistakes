use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

// Output cases moved to the cli_pkgcfg_snapshot modules.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-release-age-policy/fixture")
            .join(name),
    )
}

fn options_yaml() -> &'static str {
    r#"
permanentPackages:
  - name: acme-lib
    reason: first-party
  - name: '@acme/core'
    reason: first-party
temporarySelectors:
  - demo-temporary-package@9.9.9
scopedPrefixes:
  - '@acme/'
"#
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options_yaml()).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn files(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("pnpm-workspace.yaml"),
        root.join(".github/dependabot.yml"),
        root.join("package.json"),
        root.join("pnpm-lock.yaml"),
    ]
}

#[test]
fn missing_workspace_yaml_is_clean() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(dir.path());
    assert!(check_with_files(&root, &config(), &[]).unwrap().is_empty());
}

#[test]
fn lockfile_name_and_selector_helpers() {
    assert_eq!(
        super::lockfile::package_name_from_lock_key("@acme/core@1.0.0").as_deref(),
        Some("@acme/core")
    );
    assert_eq!(
        super::lockfile::package_name_from_lock_key("acme-lib@1.0.0").as_deref(),
        Some("acme-lib")
    );
    assert!(super::lockfile::lock_key_matches_selector(
        "demo-temporary-package@9.9.9",
        "demo-temporary-package@9.9.9"
    ));
    assert!(super::lockfile::lock_key_matches_selector(
        "demo-temporary-package@9.9.9(peer@1)",
        "demo-temporary-package@9.9.9"
    ));
}

#[test]
fn temporary_group_selector_and_timestamp_validation_is_exact() {
    assert!(super::policy::validation::is_exact_selector(
        "demo-temporary-package@9.9.9"
    ));
    assert!(super::policy::validation::is_exact_selector(
        "@acme/demo@1.0.0-beta+build"
    ));
    assert!(!super::policy::validation::is_exact_selector(
        "demo-temporary-package@^9.9.9"
    ));
    assert!(!super::policy::validation::is_exact_selector("@acme@1.0.0"));
    assert!(!super::policy::validation::is_exact_selector(
        "demo@temporary@1.0.0"
    ));
    assert!(!super::policy::validation::is_exact_selector(
        "demo-temporary-package"
    ));

    assert!(super::policy::validation::is_canonical_timestamp(
        "2027-04-30T23:59:59Z"
    ));
    assert!(super::policy::validation::is_canonical_timestamp(
        "2027-12-31T00:00:00Z"
    ));
    assert!(!super::policy::validation::is_canonical_timestamp(
        "2027-04-31T00:00:00Z"
    ));
    assert!(!super::policy::validation::is_canonical_timestamp(
        "2027-13-01T00:00:00Z"
    ));
}

#[test]
fn scan_covers_invalid_json_and_non_npm_dependabot() {
    let root = fixture("invalid-json-and-non-npm-dependabot");
    // Coverage for skip/parse arms: invalid package.json, pip Dependabot, and
    // a non-string lockfile package key. Findings are not the assertion.
    let _ = check_with_files(&root, &config(), &files(&root)).unwrap();
}

mod policy_checks;

#[path = "tests/input_shapes.rs"]
mod input_shapes;
