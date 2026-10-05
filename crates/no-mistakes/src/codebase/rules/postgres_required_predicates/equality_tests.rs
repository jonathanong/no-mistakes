use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

// Regression for #1544: an equality on a column that must be `IS NULL` selects
// only the rows the requirement excludes, so the rule keeps reporting it but
// the diagnostic explains the right fix instead.
fn run(file: &str) -> Vec<RuleFinding> {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture/fail-equality"),
    );
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "sqlInclude: [\"sql/**/*.sql\"]\nexecutorNames: []\nrelations:\n  - table: documents\n    require: [\"deleted_at IS NULL\", \"archived_by_id IS NULL\"]",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    check_with_files(&root, &config, &[root.join(file)]).unwrap()
}

#[test]
fn equality_on_required_is_null_column_is_still_reported_with_hint() {
    let findings = run("sql/archived-by.sql");
    assert_eq!(findings.len(), 1, "{findings:?}");
    let message = &findings[0].message;
    assert!(message.contains("`archived_by_id IS NULL`"), "{message}");
    assert!(
        message.contains("requireColumns: [archived_by_id]"),
        "{message}"
    );
}

#[test]
fn unrelated_query_has_no_equality_hint() {
    let findings = run("sql/unrelated.sql");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        !findings[0].message.contains("requireColumns"),
        "{findings:?}"
    );
}
