use super::*;
use crate::config::v2::{
    schema::{Project, RuleDef, RuleScope},
    NoMistakesConfig,
};

fn load_workspace(
    root: &Path,
    target_roots: &[PathBuf],
    files: &[PathBuf],
) -> anyhow::Result<crate::codebase::workspaces::WorkspaceMap> {
    let sources = crate::codebase::rules::source_store_for_files(files);
    workspace::load_workspace_with_sources(root, target_roots, files, &sources)
}

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
fn project_scoped_rule_loads_workspace_from_project_root() {
    let root = fixture_root("project-local-workspace");
    let files = package_files(
        &root,
        &[
            "frontend/pnpm-workspace.yaml",
            "frontend/packages/app/package.json",
            "frontend/packages/domain/package.json",
        ],
    );
    let mut config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    config.projects.insert(
        "frontend".to_string(),
        Project {
            root: Some("frontend".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["frontend".to_string()];

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "frontend/packages/domain/package.json");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn pnpm_root_package_is_included_in_workspace_closure() {
    let root = fixture_root("pnpm-root-package");
    let files = package_files(
        &root,
        &[
            "package.json",
            "pnpm-workspace.yaml",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/root\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/domain/package.json");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/root -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn project_scoped_package_root_loads_repository_workspace() {
    let root = fixture_root("project-package-root-workspace");
    let files = package_files(&root, &["pnpm-workspace.yaml", "packages/app/package.json"]);
    let mut config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    config.projects.insert(
        "app".to_string(),
        Project {
            root: Some("packages/app".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["app".to_string()];

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/secret")
    );
}

#[test]
fn load_workspace_defaults_to_repository_root_when_target_roots_are_empty() {
    let root = fixture_root("pass");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let workspace = load_workspace(&root, &[], &files).unwrap();
    let names: Vec<_> = workspace
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();

    assert_eq!(names, vec!["@acme/app", "workspace-root"]);
}
