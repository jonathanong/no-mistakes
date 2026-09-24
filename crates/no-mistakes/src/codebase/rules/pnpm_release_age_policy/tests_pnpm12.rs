use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn issue_root(variant: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/pnpm12-issue-1035")
            .join(variant),
    )
}

fn issue_findings(variant: &str) -> Vec<RuleFinding> {
    let root = issue_root(variant);
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "permanentPackages:\n  - name: is-odd\n    reason: direct dependency of @fixture/shared\n  - name: kind-of\n    reason: transitive-only dependency\n",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = [
        "pnpm-workspace.yaml",
        "package.json",
        "packages/api/package.json",
        "packages/shared/package.json",
        "packages/worker/package.json",
        "pnpm-lock.yaml",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect::<Vec<_>>();
    check_with_files(&root, &config, &files).unwrap()
}

#[test]
fn kind_of_is_present_in_the_project_document() {
    for variant in ["two-doc", "single-doc"] {
        let findings = issue_findings(variant);
        assert!(
            !findings
                .iter()
                .any(|finding| finding.message.contains("absent from")),
            "{variant}: {findings:?}"
        );
    }
}

#[test]
fn missing_lockfile_file_is_not_a_parse_error() {
    let root = issue_root("two-doc");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "permanentPackages:\n  - name: is-odd\n    reason: direct dependency of @fixture/shared\n  - name: kind-of\n    reason: transitive-only dependency\nlockfilePath: missing-lock.yaml\n",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = [
        "pnpm-workspace.yaml",
        "package.json",
        "packages/api/package.json",
        "packages/shared/package.json",
        "packages/worker/package.json",
        "missing-lock.yaml",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect::<Vec<_>>();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("absent from")),
        "{findings:?}"
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.message.contains("failed to parse")),
        "{findings:?}"
    );
}

#[test]
fn bad_lockfile_is_reported() {
    let malformed = issue_findings("malformed");
    assert!(
        malformed
            .iter()
            .any(|finding| finding.message.contains("failed to parse")),
        "{malformed:?}"
    );
    let non_env = issue_findings("non-env");
    assert!(
        non_env
            .iter()
            .any(|finding| finding.message.contains("non-env")),
        "{non_env:?}"
    );
}
