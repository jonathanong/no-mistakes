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
fn pnpm_lockfile_alias_resolution_name_is_forbidden() {
    let root = fixture_root("lockfile-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/secret")
    );
}

#[test]
fn pnpm_lockfile_scalar_alias_resolution_name_is_forbidden() {
    let root = fixture_root("lockfile-scalar-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/secret")
    );
}

#[test]
fn nested_lockfile_importers_are_relative_to_lockfile_directory() {
    let root = fixture_root("nested-lockfile");
    let files = package_files(
        &root,
        &[
            "pnpm-workspace.yaml",
            "frontend/pnpm-lock.yaml",
            "frontend/packages/app/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: frontend/pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "frontend/packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
}

#[test]
fn lockfile_root_importer_accepts_dot_slash_key() {
    let root = fixture_root("root-lockfile");
    let files = package_files(&root, &["package.json"]);

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/root\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "package.json");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/root -> @acme/secret")
    );
}

#[test]
fn lockfile_dependency_types_include_dev_and_optional_dependencies() {
    let root = fixture_root("lockfile-dependency-types");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret-optional\"]\ndependencyTypes: [dependencies, devDependencies, optionalDependencies]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret-optional"));
}

#[test]
fn lockfile_dependency_types_preserve_manifest_peer_dependencies() {
    let root = fixture_root("lockfile-dependency-types");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret-peer\"]\ndependencyTypes: [peerDependencies]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret-peer"));
}

#[test]
fn absolute_lockfile_path_is_supported() {
    let root = fixture_root("lockfile-alias");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);
    let lockfile = root.join("pnpm-lock.yaml");
    let yaml = format!(
        "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: {}\n",
        lockfile.display()
    );

    let findings = check_with_files(&root, &config(&yaml), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
}
