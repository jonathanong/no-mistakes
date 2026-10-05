use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

// Each fixture isolates one trigger of the reported anchoring bug; the FROM
// line is always later than the call and template start.
fn run(file: &str, suppress: bool) -> Vec<RuleFinding> {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture/fail-scoped-tagged"),
    );
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "importSpecifier: '@example/db'\nexecutorTypeNames: [TransactionQuery]\nrelations:\n  - table: documents\n    require: [\"deleted_at IS NULL\", \"owner_id IS NOT NULL\"]",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = [root.join(file)];
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    if suppress {
        let sources = super::super::source_store_for_files(&files);
        super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    }
    findings
}

fn assert_lines(file: &str, line: usize) {
    let findings = run(file, false);
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().all(|f| f.line == line), "{findings:?}");
}

#[test]
fn generic_argument_only() {
    assert_lines("src/generic-only.ts", 9);
}

#[test]
fn leading_comment_only() {
    assert_lines("src/comment-only.ts", 6);
}

#[test]
fn typed_parameter_executor_only() {
    assert_lines("src/typed-param-only.ts", 6);
}

#[test]
fn all_triggers_combined() {
    assert_lines("src/combined.ts", 9);
}

#[test]
fn directive_above_from_suppresses_the_combined_shape() {
    assert_eq!(run("src/combined-suppressed.ts", false).len(), 2);
    let findings = run("src/combined-suppressed.ts", true);
    assert!(findings.is_empty(), "{findings:?}");
}
