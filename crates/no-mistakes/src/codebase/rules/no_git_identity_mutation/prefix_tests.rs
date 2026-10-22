use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-git-identity-mutation/fixture/literal-prefixes")
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn literal_global_options_and_quoted_commands_are_reported() {
    let root = fixture();
    let findings = check_with_files(&root, &config("{}"), &[root.join("mutations.sh")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        (3..=19)
            .chain([22, 23])
            .chain(25..=29)
            .chain([31, 32, 33])
            .collect::<Vec<_>>()
    );
    assert!(findings
        .iter()
        .all(|finding| finding.rule == RULE_ID && finding.message.contains("GIT_AUTHOR_*")));
}

#[test]
fn literal_prefix_controls_remain_clear() {
    let root = fixture();
    let files = ["controls.sh", "suppressed.sh", "disabled.sh"].map(|name| root.join(name));
    // Suppression is applied by the public dispatcher, after the local scan.
    let findings =
        super::super::run_filesystem_rules_with_config(&root, &config("{}"), &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn prefixes_preserve_explicit_exclusions_and_managed_runner_exception() {
    let root = fixture();
    let files = ["mutations.sh", "managed.yml", "self-hosted.yml"].map(|name| root.join(name));
    let findings = check_with_files(
        &root,
        &config("excludePaths: ['mutations.sh']\nconditionallyAllowedWorkflows: ['*.yml']"),
        &files,
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [("self-hosted.yml", 5)]
    );
}

#[test]
fn napi_check_reports_the_same_literal_prefix_findings() {
    let root = fixture();
    let report: serde_json::Value = serde_json::from_str(
        &crate::napi_api::check_json_impl(serde_json::json!({"root": root})).unwrap(),
    )
    .unwrap();
    let findings = super::super::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(report["rules"], serde_json::to_value(findings).unwrap());
    assert_eq!(report["rules"].as_array().unwrap().len(), 28);
    assert_eq!(report["warnings"], serde_json::json!([]));
}
