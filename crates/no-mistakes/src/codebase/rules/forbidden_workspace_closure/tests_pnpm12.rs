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

fn issue_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/pnpm12-issue-1035")
            .join(name),
    )
}

fn issue_closure(variant: &str, forbidden: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let root = issue_root(variant);
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/api/package.json",
            "packages/shared/package.json",
            "packages/worker/package.json",
        ],
    );
    check_with_files(
        &root,
        &config(&format!(
            "packages: [\"@fixture/api\"]\nforbidden: [\"{forbidden}\"]\ndependencyTypes: [dependencies]\nlockfile: pnpm-lock.yaml\n"
        )),
        &files,
    )
    .unwrap()
}

#[test]
fn issue_1035_reports_is_odd_through_the_workspace_link() {
    let findings = issue_closure("two-doc", "is-odd");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@fixture/api -> @fixture/shared -> is-odd")
    );
}

#[test]
fn issue_1035_does_not_flag_is_even_for_api() {
    let findings = issue_closure("two-doc", "is-even");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn issue_1035_single_document_matches_the_two_document_closure() {
    assert_eq!(
        issue_closure("two-doc", "is-odd")[0].import,
        issue_closure("single-doc", "is-odd")[0].import
    );
}

#[test]
fn issue_1035_malformed_lockfile_is_a_finding() {
    let findings = issue_closure("malformed", "is-odd");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("could not be parsed"),
        "{findings:?}"
    );
}

#[test]
fn issue_1035_non_env_prefix_is_a_finding() {
    let findings = issue_closure("non-env", "is-odd");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("non-env"), "{findings:?}");
}

#[test]
fn env_document_with_workspace_dependencies_is_rejected() {
    // `@acme/secret` is listed on the leading `.` importer. That document is
    // not an env lockfile, so the closure reports the file instead of walking it.
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
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("non-env"), "{findings:?}");
}
