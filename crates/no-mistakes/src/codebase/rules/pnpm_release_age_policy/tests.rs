use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-release-age-policy/fixture")
            .join(name),
    )
}

fn options_yaml() -> &'static str {
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
"#
}

fn grouped_options_yaml() -> &'static str {
    r#"
permanentPackages:
  - name: acme-lib
    reason: first-party
  - name: '@acme/core'
    reason: first-party
temporaryGroups:
  - selectors:
      - demo-temporary-package@9.9.9
    reason: upstream regression pending
    eligibleForRemovalAt: '2027-01-02T03:04:05Z'
"#
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options_yaml()).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn files(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("pnpm-workspace.yaml"),
        root.join(".github/dependabot.yml"),
        root.join("package.json"),
        root.join("pnpm-lock.yaml"),
    ]
}

fn run(root: &Path) -> Vec<RuleFinding> {
    check_with_files(root, &config(), &files(root)).unwrap()
}

#[test]
fn pass_fixture_is_clean() {
    assert!(run(&fixture("pass")).is_empty());
}

#[test]
fn flags_unregistered_and_missing_excludes() {
    let findings = run(&fixture("fail-exclude"));
    let body = format!("{findings:?}");
    assert!(body.contains("unknown-package"), "{body}");
    assert!(
        body.contains("missing from minimumReleaseAgeExclude"),
        "{body}"
    );
}

#[test]
fn flags_dependabot_cooldown_miss() {
    let findings = run(&fixture("fail-dependabot"));
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("cooldown.exclude")),
        "{findings:?}"
    );
}

#[test]
fn flags_temporary_selector_missing_from_lockfile() {
    let findings = run(&fixture("fail-lockfile"));
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("absent from lockfile")),
        "{findings:?}"
    );
}

#[test]
fn empty_options_report_nothing() {
    let root = fixture("fail-exclude");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(check_with_files(&root, &config, &files(&root))
        .unwrap()
        .is_empty());
}

#[test]
fn missing_workspace_yaml_is_clean() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(dir.path());
    assert!(check_with_files(&root, &config(), &[]).unwrap().is_empty());
}

#[test]
fn malformed_workspace_yaml_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(dir.path());
    let yaml = root.join("pnpm-workspace.yaml");
    std::fs::write(&yaml, "{ invalid yaml: }}}\n").unwrap();
    let findings = check_with_files(&root, &config(), &[yaml]).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("failed to parse YAML")),
        "{findings:?}"
    );
}

#[test]
fn scoped_prefix_discovers_unregistered_active_package() {
    let root = fixture("fail-graph");
    let findings = run(&root);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("@acme/new-tool")),
        "{findings:?}"
    );
}

#[test]
fn lockfile_name_and_selector_helpers() {
    assert_eq!(
        super::lockfile::package_name_from_lock_key("@acme/core@1.0.0").as_deref(),
        Some("@acme/core")
    );
    assert_eq!(
        super::lockfile::package_name_from_lock_key("acme-lib@1.0.0").as_deref(),
        Some("acme-lib")
    );
    assert!(super::lockfile::lock_key_matches_selector(
        "demo-temporary-package@9.9.9",
        "demo-temporary-package@9.9.9"
    ));
    assert!(super::lockfile::lock_key_matches_selector(
        "demo-temporary-package@9.9.9(peer@1)",
        "demo-temporary-package@9.9.9"
    ));
}

#[test]
fn temporary_group_selector_and_timestamp_validation_is_exact() {
    assert!(super::policy::validation::is_exact_selector(
        "demo-temporary-package@9.9.9"
    ));
    assert!(super::policy::validation::is_exact_selector(
        "@acme/demo@1.0.0-beta+build"
    ));
    assert!(!super::policy::validation::is_exact_selector(
        "demo-temporary-package@^9.9.9"
    ));
    assert!(!super::policy::validation::is_exact_selector("@acme@1.0.0"));
    assert!(!super::policy::validation::is_exact_selector(
        "demo@temporary@1.0.0"
    ));
    assert!(!super::policy::validation::is_exact_selector(
        "demo-temporary-package"
    ));

    assert!(super::policy::validation::is_canonical_timestamp(
        "2027-04-30T23:59:59Z"
    ));
    assert!(super::policy::validation::is_canonical_timestamp(
        "2027-12-31T00:00:00Z"
    ));
    assert!(!super::policy::validation::is_canonical_timestamp(
        "2027-04-31T00:00:00Z"
    ));
    assert!(!super::policy::validation::is_canonical_timestamp(
        "2027-13-01T00:00:00Z"
    ));
}

#[test]
fn grouped_temporary_selectors_are_flattened_for_existing_drift_checks() {
    let root = fixture("pass");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(grouped_options_yaml()).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(check_with_files(&root, &config, &files(&root))
        .unwrap()
        .is_empty());
}

#[test]
fn invalid_temporary_group_metadata_is_reported_without_using_its_selectors() {
    let root = fixture("pass");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                r#"
temporaryGroups:
  - selectors: []
    reason: ''
    eligibleForRemovalAt: '2027-01-02T03:04:05+00:00'
  - selectors:
      - demo-temporary-package@^9.9.9
    reason: stale selector
    eligibleForRemovalAt: '2027-02-30T03:04:05Z'
  - selectors:
      - demo-temporary-package@9.9.9
    reason: duplicate in invalid group
    eligibleForRemovalAt: invalid
temporarySelectors:
  - demo-temporary-package@9.9.9
"#,
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let body = format!(
        "{:?}",
        check_with_files(&root, &config, &files(&root)).unwrap()
    );
    assert!(body.contains("selectors must contain"), "{body}");
    assert!(body.contains("reason must be non-empty"), "{body}");
    assert!(body.contains("canonical YYYY-MM-DDTHH:mm:ssZ"), "{body}");
    assert!(
        body.contains("duplicates another temporary selector"),
        "{body}"
    );
    assert!(!body.contains("absent from lockfile"), "{body}");
}

#[test]
fn duplicate_temporary_selectors_across_flat_and_grouped_configuration_are_reported() {
    let root = fixture("pass");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                r#"
temporarySelectors:
  - demo-temporary-package@9.9.9
temporaryGroups:
  - selectors:
      - demo-temporary-package@9.9.9
    reason: duplicate
    eligibleForRemovalAt: '2027-01-02T03:04:05Z'
  - selectors:
      - demo-temporary-package@9.9.9
    reason: duplicate across groups
    eligibleForRemovalAt: '2027-01-03T03:04:05Z'
"#,
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let body = format!(
        "{:?}",
        check_with_files(&root, &config, &files(&root)).unwrap()
    );
    assert!(
        body.contains("duplicates another temporary selector"),
        "{body}"
    );
}

#[test]
fn past_eligibility_date_is_audit_metadata_not_a_ci_failure() {
    let root = fixture("pass");
    let config = NoMistakesConfig {
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
temporaryGroups:
  - selectors:
      - demo-temporary-package@9.9.9
    reason: expired audit entry
    eligibleForRemovalAt: '2000-02-29T00:00:00Z'
"#,
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(check_with_files(&root, &config, &files(&root))
        .unwrap()
        .is_empty());
}

#[test]
fn policy_checks_report_each_cross_file_drift_kind() {
    let opts = Options {
        permanent_packages: vec![
            super::PermanentPackage {
                name: "@acme/covered".to_string(),
                reason: "first-party".to_string(),
            },
            super::PermanentPackage {
                name: "missing-permanent".to_string(),
                reason: "first-party".to_string(),
            },
        ],
        temporary_selectors: vec![
            "temporary-present@1.0.0".to_string(),
            "temporary-missing@2.0.0".to_string(),
        ],
        scoped_prefixes: vec!["@acme/".to_string()],
        ..Default::default()
    };
    let snapshot = super::policy::Snapshot {
        exclude: vec![
            super::policy::ExcludeEntry::Name("@acme/covered".to_string()),
            super::policy::ExcludeEntry::Name("temporary-present@1.0.0".to_string()),
            super::policy::ExcludeEntry::Name("unknown-package".to_string()),
            super::policy::ExcludeEntry::Name("unknown-package".to_string()),
            super::policy::ExcludeEntry::Other,
        ],
        cooldown: Some(vec![
            super::policy::CooldownEntry::Pattern("@acme/**".to_string()),
            super::policy::CooldownEntry::Other,
        ]),
        active_names: HashSet::from(["@acme/covered".to_string(), "@acme/new-tool".to_string()]),
        lockfile_keys: Some(vec!["temporary-present@1.0.0".to_string()]),
    };

    let messages = super::policy::check(&opts, &snapshot)
        .into_iter()
        .map(|issue| issue.message)
        .collect::<Vec<_>>();
    let body = messages.join("\n");
    assert!(body.contains("unknown-package\" duplicates"), "{body}");
    assert!(
        body.contains("unknown-package\" is not in a release-age exemption registry"),
        "{body}"
    );
    assert!(
        body.contains("missing-permanent\" is missing from minimumReleaseAgeExclude"),
        "{body}"
    );
    assert!(
        body.contains("temporary-missing@2.0.0\" is missing from minimumReleaseAgeExclude"),
        "{body}"
    );
    assert!(
        body.contains("cooldown.exclude\" must be a string glob pattern"),
        "{body}"
    );
    assert!(
        body.contains("missing-permanent\" is not covered by npm cooldown.exclude"),
        "{body}"
    );
    assert!(
        body.contains(
            "@acme/new-tool\" is an active first-party package missing from permanentPackages"
        ),
        "{body}"
    );
    assert!(
        body.contains(
            "missing-permanent\" is registered but absent from package manifests and the lockfile"
        ),
        "{body}"
    );
    assert!(
        body.contains("temporary-missing@2.0.0\" is absent from lockfile packages"),
        "{body}"
    );
}

#[test]
fn invalid_dependabot_globs_do_not_cover_permanent_packages() {
    let opts = Options {
        permanent_packages: vec![super::PermanentPackage {
            name: "acme-lib".to_string(),
            reason: "first-party".to_string(),
        }],
        ..Default::default()
    };
    let snapshot = super::policy::Snapshot {
        exclude: vec![super::policy::ExcludeEntry::Name("acme-lib".to_string())],
        cooldown: Some(vec![super::policy::CooldownEntry::Pattern("[".to_string())]),
        active_names: HashSet::from(["acme-lib".to_string()]),
        lockfile_keys: None,
    };

    let messages = super::policy::check(&opts, &snapshot)
        .into_iter()
        .map(|issue| issue.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("acme-lib\" is not covered by npm cooldown.exclude")),
        "{messages:#?}"
    );
}
