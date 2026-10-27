use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::{BTreeMap, BTreeSet};

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-workspace-closure")
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

fn package_files(root: &Path, files: &[&str]) -> Vec<PathBuf> {
    files.iter().map(|file| root.join(file)).collect()
}

#[test]
fn invalid_glob_pattern_emits_config_finding() {
    let root = fixture_root("pass");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"[\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("invalid glob pattern"));
}

#[test]
fn unknown_configured_package_emits_config_finding() {
    let root = fixture_root("pass");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/missing\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0]
        .message
        .contains("not a named workspace package"));
}

#[test]
fn traversal_tolerates_duplicate_and_missing_workspace_nodes() {
    let root = fixture_root("pass");
    let mut nodes = BTreeMap::new();
    nodes.insert(
        "@acme/app".to_string(),
        PackageNode {
            manifest: root.join("packages/app/package.json"),
            deps: vec![
                Dependency {
                    name: "@acme/missing".to_string(),
                    resolved_name: None,
                    workspace_name: Some("@acme/missing".to_string()),
                    field: "dependencies".to_string(),
                },
                Dependency {
                    name: "@acme/missing".to_string(),
                    resolved_name: None,
                    workspace_name: Some("@acme/missing".to_string()),
                    field: "dependencies".to_string(),
                },
            ],
        },
    );
    let workspace_names = BTreeSet::from(["@acme/app".to_string(), "@acme/missing".to_string()]);
    let forbidden = scan::build_globset(&["@acme/secret".to_string()]).unwrap();
    let config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    let source_filter =
        crate::codebase::rules::path_filter::RulePathFilter::new(&root, &config, &config.rules[0])
            .unwrap();
    let mut findings = Vec::new();

    let traversal = traversal::Traversal::new(
        &root,
        &nodes,
        &workspace_names,
        &forbidden,
        &source_filter,
        None,
    );
    traversal.collect_findings_for_package("@acme/app", &mut findings);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn traversal_tolerates_missing_start_node() {
    let root = fixture_root("pass");
    let nodes = BTreeMap::new();
    let workspace_names = BTreeSet::from(["@acme/app".to_string()]);
    let forbidden = scan::build_globset(&["@acme/secret".to_string()]).unwrap();
    let config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    let source_filter =
        crate::codebase::rules::path_filter::RulePathFilter::new(&root, &config, &config.rules[0])
            .unwrap();
    let mut findings = Vec::new();

    let traversal = traversal::Traversal::new(
        &root,
        &nodes,
        &workspace_names,
        &forbidden,
        &source_filter,
        None,
    );
    traversal.collect_findings_for_package("@acme/app", &mut findings);

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn normalize_importer_path_keeps_workspace_root_as_dot() {
    assert_eq!(lockfile::normalize_importer_path("./"), ".");
}

#[test]
fn missing_options_emit_config_finding() {
    let root = fixture_root("pass");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("packages"));
    assert!(findings[0].message.contains("forbidden"));
}
