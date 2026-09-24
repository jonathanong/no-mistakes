use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/forbidden-workspace-closure")
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
fn project_importer_forbidden_dependency_is_reported() {
    let root = fixture_root("pnpm12-two-doc");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);
    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/secret")
    );
}

#[test]
fn env_importer_is_not_a_workspace_edge() {
    // `@acme/secret` is a dependency of the env `.` importer only.
    let root = fixture_root("pnpm12-env-only-forbidden");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);
    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
