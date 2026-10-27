use super::workspace::normalize_entry;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/package-json-nested-workspace-coverage")
            .join(name),
    )
}
fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}
fn files(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let app_manifest = root.join("apps/package.json");
    if app_manifest.exists() {
        paths.push(app_manifest);
    }
    if root.join("apps/api/package.json").exists() {
        paths.push(root.join("apps/api/package.json"));
    }
    for package in ["utils", "unused", "signing"] {
        let path = root.join("packages").join(package).join("package.json");
        if path.exists() {
            paths.push(path);
        }
    }
    let deploy_environment = root.join("ts-shared/deploy-environment/package.json");
    if deploy_environment.exists() {
        paths.push(deploy_environment);
    }
    for path in [
        root.join("web/package.json"),
        root.join("web-tools/package.json"),
    ] {
        if path.exists() {
            paths.push(path);
        }
    }
    for path in [
        root.join("lambdas/image/package.json"),
        root.join("lambdas/image/task/package.json"),
    ] {
        if path.exists() {
            paths.push(path);
        }
    }
    paths
}

// Finding cases moved to cli_pkgcfg_snapshot_package_json_nested_workspace_coverage.

#[test]
fn normalizes_workspace_entries_without_collapsing_unresolved_parent_segments() {
    assert_eq!(
        normalize_entry("../../ts-shared/deploy-environment"),
        "../../ts-shared/deploy-environment"
    );
    assert_eq!(
        normalize_entry("./../../ts-shared/unused/../deploy-environment"),
        "../../ts-shared/deploy-environment"
    );
    assert_eq!(
        normalize_entry(r"..\..\ts-shared\deploy-environment"),
        "../../ts-shared/deploy-environment"
    );
}

#[test]
fn preserves_wildcard_parent_traversal_that_cannot_be_normalized_lexically() {
    assert_eq!(
        normalize_entry("../packages/*/../utils"),
        "../packages/*/../utils"
    );
}

#[test]
fn invalid_root_globs_fail_closed() {
    let root = fixture_root("valid");
    let error = check_with_files(
        &root,
        &config("roots: ['[']\ndependencyNamePrefixes: ['@shared/']\n"),
        &files(&root),
    )
    .expect_err("invalid root glob must fail");
    assert!(
        error.to_string().contains("error parsing glob")
            || error.to_string().contains("invalid")
            || error.to_string().contains("["),
        "{error:#}"
    );
}
