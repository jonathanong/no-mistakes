use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-step-timeouts/fixture")
            .join(scenario),
    )
}

fn run_files(scenario: &str, options: &str, files: &[&str]) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = files.iter().map(|file| root.join(file)).collect::<Vec<_>>();
    check_with_files(&root, &config, &files).unwrap()
}

// Output cases moved to cli_gha_snapshot_step_timeouts.

#[test]
fn deferred_suppression_retains_findings_for_audit_reports() {
    for scenario in [
        "disable-file",
        "disable-line",
        "parallel-suppressed",
        "commented-job-suppressed",
    ] {
        let root = fixture(scenario);
        let files = [root.join(".github/workflows/ci.yml")];
        let sources = super::super::source_store_for_files(&files);
        let config = NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.into(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str("{}").unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let findings =
            check_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
        assert_eq!(findings.len(), 1, "{scenario}: {findings:?}");
    }
}

mod locations;

#[test]
fn allow_without_override_still_enforces_the_global_cap() {
    let findings = run_files(
        "action-caps",
        r#"
maxMinutes: 5
onlyConfiguredActions: true
actionCaps:
  - uses: actions/setup-node
    maxMinutes: 5
allow:
  - step: .github/workflows/ci.yml#build#dynamic-slow
    reason: pinned upstream
"#,
        &[".github/workflows/ci.yml"],
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("over the 5-minute cap")),
        "{findings:?}"
    );
}

#[test]
fn equal_allow_cap_counts_as_used_when_it_suppresses_a_dynamic_action() {
    let findings = run_files(
        "action-caps",
        r#"
maxMinutes: 10
actionCaps:
  - uses: actions/setup-node
    maxMinutes: 5
allow:
  - step: .github/workflows/ci.yml#build#dynamic-slow
    reason: pinned upstream
    maxMinutes: 10
"#,
        &[".github/workflows/ci.yml"],
    );
    assert!(
        findings.iter().all(|finding| !finding.message.contains("stale")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("dynamic action")),
        "{findings:?}"
    );
}

#[test]
fn allowances_for_unselected_workflows_are_not_stale() {
    let findings = run_files(
        "fail",
        "allow: [{step: '.github/workflows/excluded.yml#test#1', reason: legacy}]",
        &[],
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn action_caps_validate_invalid_options() {
    let root = fixture("pass");
    let files = [root.join(".github/workflows/ci.yml")];
    for options in [
        "actionCaps: [{uses: '', maxMinutes: 5}]",
        "actionCaps: [{uses: '${{ inputs.action }}', maxMinutes: 5}]",
        "actionCaps: [{uses: 'actions/setup-node', maxMinutes: 0}]",
        "actionCaps: [{uses: 'actions/setup-node', maxMinutes: 361}]",
        "actionCaps: [{uses: 'Actions/Setup-Node@v4', maxMinutes: 5}, {uses: 'actions/setup-node@v5', maxMinutes: 3}]",
        "onlyConfiguredActions: true",
        "unknownField: 1",
    ] {
        let config = NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.into(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str(options).unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            check_with_files(&root, &config, &files).is_err(),
            "expected config error for `{options}`"
        );
    }
}

#[test]
fn dynamic_uses_allow_entries_enforce_reason_and_max_minutes() {
    let findings = run_files(
        "action-caps",
        r#"
onlyConfiguredActions: true
actionCaps:
  - uses: actions/setup-node
    maxMinutes: 5
allow:
  - step: .github/workflows/ci.yml#build#dynamic-slow
    reason: ""
    maxMinutes: 5
  - step: .github/workflows/ci.yml#build#dynamic-fast
    reason: "bounded setup"
    maxMinutes: 5
"#,
        &[".github/workflows/ci.yml"],
    );
    assert!(
        findings
            .iter()
            .any(|f| f.message.contains("needs a reason") && f.message.contains("dynamic-slow")),
        "expected missing reason finding, got {findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.message.contains("over the 5-minute cap")
                && f.message.contains("dynamic-slow")),
        "expected cap finding for dynamic-slow, got {findings:?}"
    );
    assert!(
        !findings.iter().any(|f| f.message.contains("dynamic-fast")),
        "dynamic-fast should pass within its allow cap, got {findings:?}"
    );

    let unallowed = run_files(
        "action-caps",
        r#"
onlyConfiguredActions: true
unanalyzable: report
actionCaps:
  - uses: actions/setup-node
    maxMinutes: 5
  - uses: Owner/Repo/SubPath@v1
    maxMinutes: 3
  - uses: bare-action
    maxMinutes: 2
"#,
        &[".github/workflows/ci.yml"],
    );
    assert!(
        unallowed
            .iter()
            .any(|f| f.message.contains("uses dynamic action")),
        "expected unallowed dynamic-uses report, got {unallowed:?}"
    );

    let bad_yaml = run_files(
        "bad-yaml",
        "allow: [{step: '.github/workflows/ci.yml#a#1', reason: dup}, {step: '.github/workflows/ci.yml#a#1', reason: dup}]",
        &[".github/workflows/ci.yml"],
    );
    assert!(
        bad_yaml.iter().any(|f| f.message.contains("invalid YAML")),
        "expected invalid YAML finding, got {bad_yaml:?}"
    );
    assert!(
        bad_yaml.iter().any(|f| f.message.contains("duplicate")),
        "expected duplicate allow finding, got {bad_yaml:?}"
    );

    let without_cap = run_files(
        "action-caps",
        "onlyConfiguredActions: true\nactionCaps: [{uses: 'actions/setup-node', maxMinutes: 5}]\nallow: [{step: '.github/workflows/ci.yml#build#dynamic-fast', reason: 'allowed'}]\n",
        &[".github/workflows/ci.yml"],
    );
    assert!(
        !without_cap
            .iter()
            .any(|f| f.message.contains("dynamic-fast")),
        "{without_cap:?}"
    );

    let compiled = options::compile_options(
        serde_yaml::from_str("actionCaps: [{uses: 'actions/setup-node', maxMinutes: 5}]").unwrap(),
        crate::config::v2::Unanalyzable::Advise,
        false,
    )
    .unwrap();
    let num_uses: serde_yaml::Mapping = serde_yaml::from_str("uses: 42").unwrap();
    assert!(matches!(
        compiled.match_action_cap(&num_uses),
        options::UsesMatch::Dynamic { .. }
    ));
    let empty_uses: serde_yaml::Mapping = serde_yaml::from_str("uses: ''").unwrap();
    assert!(matches!(
        compiled.match_action_cap(&empty_uses),
        options::UsesMatch::None
    ));
}
