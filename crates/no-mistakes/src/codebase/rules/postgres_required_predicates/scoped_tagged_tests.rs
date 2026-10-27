// Output cases moved to cli_postgres_snapshot_required_predicates.
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

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

#[test]
fn directive_above_from_suppresses_the_combined_shape() {
    assert_eq!(run("src/combined-suppressed.ts", false).len(), 2);
    let findings = run("src/combined-suppressed.ts", true);
    assert!(findings.is_empty(), "{findings:?}");
}
