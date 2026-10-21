use super::alias::catalog_workspace_name;
use super::*;
use crate::codebase::workspaces::{CatalogHit, CatalogIndex, WorkspacePackage};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-workspace-closure")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n")
            .unwrap(),
        ..Default::default()
    });
    config
}

fn findings(name: &str) -> Vec<RuleFinding> {
    let root = fixture_root(name);
    let mut files = vec![
        root.join("package.json"),
        root.join("pnpm-workspace.yaml"),
        root.join("packages/app/package.json"),
        root.join("packages/domain/package.json"),
    ];
    let included = root.join("pnpm/catalog.yaml");
    if included.is_file() {
        files.push(included);
    }
    check_with_files(&root, &config(), &files).unwrap()
}

fn assert_secret_closure(findings: &[RuleFinding]) {
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "packages/domain/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn catalog_workspace_protocol_reaches_the_forbidden_package() {
    assert_secret_closure(&findings("catalog-workspace"));
}

#[test]
fn semver_catalog_entries_are_not_walked() {
    let findings = findings("catalog-semver");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn catalog_from_a_local_include_is_walked() {
    assert_secret_closure(&findings("catalog-included"));
}

#[test]
fn duplicate_catalog_settings_are_not_walked() {
    let findings = findings("catalog-duplicate");
    assert!(
        findings.iter().any(|finding| {
            finding.file == "pnpm-workspace.yaml"
                && finding
                    .message
                    .contains("setting \"catalog\" is claimed by more than one pnpm source")
        }),
        "{findings:?}"
    );
    assert!(
        findings.iter().all(|finding| finding.import.is_none()),
        "{findings:?}"
    );
}

fn workspace() -> crate::codebase::workspaces::WorkspaceMap {
    crate::codebase::workspaces::WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/domain".to_string(),
        dir: PathBuf::from("/repo/packages/domain"),
        entry: None,
        exports: None,
        imports: None,
    }])
}

fn catalogs(specifier: &str, base: &str) -> CatalogIndex {
    let mut default_entries = BTreeMap::new();
    default_entries.insert(
        "@acme/domain".to_string(),
        CatalogHit {
            specifier: specifier.to_string(),
            base_dir: PathBuf::from(base),
        },
    );
    CatalogIndex {
        default_entries,
        named: BTreeMap::new(),
    }
}

fn resolved(protocol: &str, specifier: &str, base: &str) -> Option<String> {
    catalog_workspace_name(
        "@acme/domain",
        protocol,
        &workspace(),
        &catalogs(specifier, base),
    )
}

#[test]
fn catalog_protocols_match_direct_workspace_dependencies() {
    assert_eq!(
        resolved("catalog:", "workspace:*", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "workspace:~", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "workspace:^", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:default", "workspace:*", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "workspace:../packages/domain", "/repo/pnpm"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "file:../packages/domain", "/repo/pnpm"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "link:./domain", "/repo/packages"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(resolved("catalog:", "^1.0.0", "/repo"), None);
    assert_eq!(resolved("catalog:", "workspace:^1.2.3", "/repo"), None);
    assert_eq!(resolved("catalog:", "file:/etc/domain", "/repo"), None);
    assert_eq!(
        resolved("catalog:", "link:https://example.test", "/repo"),
        None
    );
    assert_eq!(resolved("catalog:foo/bar", "workspace:*", "/repo"), None);
    assert_eq!(resolved("catalog:foo bar", "workspace:*", "/repo"), None);
    assert_eq!(resolved("catalog:foo\\bar", "workspace:*", "/repo"), None);
    assert_eq!(
        catalog_workspace_name(
            "@acme/missing",
            "catalog:",
            &workspace(),
            &catalogs("workspace:*", "/repo")
        ),
        None
    );
    let mut named = BTreeMap::new();
    named.insert(
        "@acme/domain".to_string(),
        CatalogHit {
            specifier: "workspace:^".to_string(),
            base_dir: PathBuf::from("/repo"),
        },
    );
    let index = CatalogIndex {
        default_entries: BTreeMap::new(),
        named: BTreeMap::from([("legacy".to_string(), named)]),
    };
    assert_eq!(
        catalog_workspace_name("@acme/domain", "catalog:legacy", &workspace(), &index),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        catalog_workspace_name("@acme/domain", "catalog:missing", &workspace(), &index),
        None
    );
}
