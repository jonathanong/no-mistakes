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
