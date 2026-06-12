use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

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

#[test]
fn reports_package_json_under_configured_roots_missing_from_workspaces() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/admin/package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/web/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config("packageRoots: [./packages]\nrequireNamedPackage: true\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].file, "packages/admin/package.json");
    assert_eq!(findings[1].file, "packages/web/package.json");
    assert!(findings[0].message.contains("not covered"));
}

#[test]
fn allowlist_suppresses_known_non_workspace_packages() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/web/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            "packageRoots: [packages]\nallowlist: [packages/web/package.json]\nrequireNamedPackage: true\n",
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn allowlist_normalizes_relative_package_json_paths() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/web/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            "packageRoots: [packages]\nallowlist: [./packages/web/package.json]\nrequireNamedPackage: true\n",
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn skips_non_package_files_and_unnamed_packages_when_required() {
    let root = fixture_root("missing");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/unnamed/package.json"),
        root.join("packages/unnamed/index.ts"),
    ];
    let findings = check_with_files(
        &root,
        &config("packageRoots: [packages]\nrequireNamedPackage: true\n"),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    assert_eq!(
        package_name(&root.join("packages/unnamed/package.json")),
        None
    );
    assert_eq!(
        package_name(&fixture_root("invalid-package-json").join("packages/bad/package.json")),
        None
    );
}

#[test]
fn covered_unnamed_packages_pass_when_names_are_not_required() {
    let root = fixture_root("unnamed-covered");
    let files = vec![
        root.join("package.json"),
        root.join("packages/unnamed/package.json"),
    ];
    let findings = check_with_files(&root, &config("packageRoots: [packages]\n"), &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
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
    assert!(exclude.is_match("packages/internal"));
    assert!(!exclude.is_match("packages/web"));
}
