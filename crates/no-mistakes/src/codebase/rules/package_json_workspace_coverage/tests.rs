use super::patterns::{build_workspace_globset, normalize_relative_path};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn package_name(path: &Path) -> Option<String> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    super::coverage_scope::package_name_with_sources(path, &sources)
}

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/package-json-workspace-coverage")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

// Finding cases moved to cli_pkgcfg_snapshot_package_json_workspace_coverage.

#[test]
fn unnamed_manifest_package_name_is_absent() {
    let root = fixture_root("missing");
    assert_eq!(
        package_name(&root.join("packages/unnamed/package.json")),
        None
    );
}

#[test]
fn malformed_json_package_name_is_absent() {
    let root = fixture_root("invalid-package-json");
    assert_eq!(package_name(&root.join("packages/bad/package.json")), None);
}

#[test]
fn workspace_globset_builder_splits_includes_and_excludes() {
    let patterns = vec![
        "./packages/*".to_string(),
        "!./packages/internal".to_string(),
    ];
    let include = build_workspace_globset(&patterns, false).unwrap();
    let exclude = build_workspace_globset(&patterns, true).unwrap();

    assert!(include.is_match("packages/web"));
    assert!(include.is_match("packages/internal"));
    assert!(!include.is_match("packages/group/app"));
    assert!(exclude.is_match("packages/internal"));
    assert!(!exclude.is_match("packages/web"));
    assert!(build_workspace_globset(&["[".to_string()], false).is_err());
    assert_eq!(
        normalize_relative_path("./packages/../packages/web"),
        "packages/web"
    );
    assert_eq!(
        normalize_relative_path("*/../packages/*"),
        "*/../packages/*"
    );
}

fn files_named(root: &Path, names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(|name| root.join(name)).collect()
}

// Sibling modules keep this file under the test line cap.
#[path = "tests_coverage.rs"]
mod tests_coverage;
#[path = "tests_patterns.rs"]
mod tests_patterns;
