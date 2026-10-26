use super::included_exclude::included_exclude;
use super::policy::{check_with_complete_names as policy_check, ExcludeEntry, Snapshot};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-release-age-policy/fixture")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                r#"
permanentPackages:
  - name: acme-lib
    reason: first-party
  - name: '@acme/core'
    reason: first-party
temporarySelectors:
  - demo-temporary-package@9.9.9
scopedPrefixes:
  - '@acme/'
"#,
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("pnpm-workspace.yaml"),
        root.join(".github/dependabot.yml"),
        root.join("package.json"),
        root.join("pnpm-lock.yaml"),
    ];
    let included = root.join("pnpm/age.yaml");
    if included.is_file() {
        files.push(included);
    }
    files
}

fn run(name: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    check_with_files(&root, &config(), &files(&root)).unwrap()
}

#[test]
fn included_exclude_list_matches_the_root_list() {
    let findings = run("included-exclude");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn duplicate_exclude_does_not_guess_a_list() {
    let findings = run("duplicate-exclude");
    let body = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        body.contains(
            "setting \"minimumReleaseAgeExclude\" is claimed by more than one pnpm source"
        ),
        "{body}"
    );
    assert!(
        !body.contains("missing from minimumReleaseAgeExclude"),
        "{body}"
    );
    assert!(!body.contains("other-pkg"), "{body}");
}

#[test]
fn unreadable_exclude_reports_the_load_error_and_the_missing_entries() {
    let findings = run("unreadable-exclude");
    let body = format!("{findings:?}");
    assert!(body.contains("could not be read"), "{body}");
    assert!(body.contains("acme-lib"), "{body}");
    assert!(
        body.contains("missing from minimumReleaseAgeExclude"),
        "{body}"
    );
    assert!(
        !body.contains("claimed by more than one pnpm source"),
        "{body}"
    );
}

#[test]
fn conflict_entries_skip_exclude_drift() {
    let opts = Options {
        permanent_packages: vec![PermanentPackage {
            name: "acme-lib".to_string(),
            reason: "first-party".to_string(),
        }],
        ..Default::default()
    };
    let snapshot = Snapshot {
        exclude: vec![
            ExcludeEntry::Conflict,
            ExcludeEntry::Name("other".to_string()),
        ],
        cooldown: None,
        active_names: HashSet::new(),
        lockfile_keys: None,
    };
    let messages = policy_check(&opts, &snapshot, true)
        .into_iter()
        .map(|issue| issue.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("minimumReleaseAgeExclude")
                && !message.contains("other")),
        "{messages:?}"
    );
}

#[test]
fn unreadable_workspace_keeps_the_fallback_exclude_list() {
    let root = fixture("non-mapping-workspace");
    let path = root.join("pnpm-workspace.yaml");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let fallback = vec![ExcludeEntry::Name("kept".to_string())];
    let (entries, notes) = included_exclude(&root, &path, &sources, fallback);
    assert!(notes.is_empty());
    assert!(matches!(entries.first(), Some(ExcludeEntry::Name(name)) if name == "kept"));
    let (entries, notes) = included_exclude(
        Path::new("/repo"),
        Path::new("/"),
        &sources,
        vec![ExcludeEntry::Name("kept".to_string())],
    );
    assert!(notes.is_empty());
    assert!(matches!(entries.first(), Some(ExcludeEntry::Name(name)) if name == "kept"));
}
