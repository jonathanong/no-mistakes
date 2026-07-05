use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

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
fn unsupported_lockfile_emits_config_finding() {
    let root = fixture_root("lockfile-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: package-lock.json\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("pnpm-lock.yaml only"));
}

#[test]
fn unreadable_lockfile_emits_config_finding() {
    let root = fixture_root("lockfile-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: missing/pnpm-lock.yaml\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("could not read lockfile"));
}

#[test]
fn lockfile_without_importers_emits_config_finding() {
    let root = fixture_root("lockfile-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: no-importers/pnpm-lock.yaml\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("has no pnpm importers"));
}

#[test]
fn lockfile_missing_workspace_importer_emits_config_finding() {
    let root = fixture_root("lockfile-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: missing-importer/pnpm-lock.yaml\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("missing importer"));
}

#[test]
fn lockfile_dependency_types_reject_unknown_dependency_type() {
    let root = fixture_root("lockfile-dependency-types");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret-peer\"]\ndependencyTypes: [bundledDependencies]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0]
        .message
        .contains("unsupported dependency type 'bundledDependencies'"));
}
