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

fn findings(name: &str, options: &str) -> Vec<RuleFinding> {
    let root = fixture_root(name);
    let files = files(&root);
    check_with_files(&root, &config(options), &files).unwrap()
}
const OPTIONS: &str = "roots: [apps, lambdas/*]\ndependencyNamePrefixes: ['@shared/']\n";

#[test]
fn accepts_explicit_workspace_paths_for_root_and_nested_package_dependencies() {
    assert!(findings("valid", OPTIONS).is_empty());
}
#[test]
fn accepts_consecutive_parent_segments_in_nested_workspace_paths() {
    assert!(findings("parent-path", OPTIONS).is_empty());
}
#[test]
fn excludes_sibling_directories_that_only_share_the_workspace_name_prefix() {
    assert!(findings(
        "sibling-prefix",
        "roots: [web]\ndependencyNamePrefixes: ['@shared/']\n"
    )
    .is_empty());
}
#[test]
fn reports_explicit_workspace_entries_without_a_matching_dependency() {
    let findings = findings("extra", OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("unused nested workspace entries"));
}
#[test]
fn reports_matching_dependencies_missing_from_the_root_workspace_array() {
    let findings = findings("missing", OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("missing nested workspace entries"));
}
#[test]
fn rejects_wildcards_that_cover_configured_dependency_packages() {
    let findings = findings("glob", OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("uses a wildcard"));
}
#[test]
fn rejects_brace_globs_that_cover_configured_dependency_packages() {
    let findings = findings("brace-glob", OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("uses a wildcard"));
}
#[test]
fn expands_configured_root_globs_for_independent_nested_workspaces() {
    let findings = findings("lambda", OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "lambdas/image/package.json");
}
#[test]
fn skips_absent_configured_roots_in_partial_checkouts() {
    assert!(findings(
        "partial",
        "roots: [apps, absent]\ndependencyNamePrefixes: ['@shared/']\n"
    )
    .is_empty());
}
#[test]
fn fails_closed_when_a_matching_dependency_has_no_visible_target_manifest() {
    let findings = findings("unresolved", OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("no unique visible package.json target"));
}

#[test]
fn normalizes_workspace_entries_without_collapsing_unresolved_parent_segments() {
    assert_eq!(
        normalize_workspace_entry("../../ts-shared/deploy-environment"),
        "../../ts-shared/deploy-environment"
    );
    assert_eq!(
        normalize_workspace_entry("./../../ts-shared/unused/../deploy-environment"),
        "../../ts-shared/deploy-environment"
    );
    assert_eq!(
        normalize_workspace_entry(r"..\..\ts-shared\deploy-environment"),
        "../../ts-shared/deploy-environment"
    );
}

#[test]
fn preserves_wildcard_parent_traversal_that_cannot_be_normalized_lexically() {
    assert_eq!(
        normalize_workspace_entry("../packages/*/../utils"),
        "../packages/*/../utils"
    );
}
