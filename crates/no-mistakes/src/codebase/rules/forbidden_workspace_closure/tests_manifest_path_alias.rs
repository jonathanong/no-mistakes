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
fn file_path_alias_extends_closure() {
    let root = fixture_root("manifest-file-path-alias");
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
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
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
fn root_relative_file_path_alias_extends_closure() {
    let root = fixture_root("manifest-root-file-path-alias");
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
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/root -> @acme/domain -> @acme/secret")
    );
}
