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

fn assert_lockfile_closure(fixture: &str) {
    let root = fixture_root(fixture);
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
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
fn pnpm_lockfile_workspace_alias_extends_closure() {
    assert_lockfile_closure("lockfile-workspace-alias");
}

#[test]
fn pnpm_lockfile_exact_workspace_range_uses_dependency_name() {
    assert_lockfile_closure("lockfile-exact-workspace-range");
}

#[test]
fn pnpm_lockfile_prerelease_workspace_range_uses_dependency_name() {
    assert_lockfile_closure("lockfile-prerelease-workspace-range");
}

#[test]
fn pnpm_lockfile_x_workspace_range_uses_dependency_name() {
    assert_lockfile_closure("lockfile-x-workspace-range");
}

#[test]
fn pnpm_lockfile_digit_prefixed_aliases_resolve_package_names() {
    let root = fixture_root("lockfile-digit-alias");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let direct = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"7zip-bin\"]\nlockfile: pnpm-lock.yaml\n"),
        &files,
    )
    .unwrap();

    assert_eq!(direct.len(), 1);
    assert_eq!(direct[0].target.as_deref(), Some("7zip-bin"));
    assert_eq!(direct[0].import.as_deref(), Some("@acme/app -> 7zip-bin"));

    let closure = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert_eq!(closure.len(), 1);
    assert_eq!(
        closure[0].import.as_deref(),
        Some("@acme/app -> 3d-domain -> @acme/secret")
    );
}

#[test]
fn pnpm_lockfile_workspace_path_alias_extends_closure() {
    assert_lockfile_closure("lockfile-path-alias");
}

#[test]
fn pnpm_lockfile_registry_dependency_name_does_not_extend_closure() {
    let root = fixture_root("lockfile-registry-same-name");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\nlockfile: pnpm-lock.yaml\n",
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}
