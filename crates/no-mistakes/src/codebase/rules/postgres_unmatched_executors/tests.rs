use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

const RULE: &str = "postgres-lock-ordering";

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
        ..Default::default()
    });
    config
}

// Output cases moved to cli_postgres_snapshot_unmatched_executors.
// production_factory stays: dynamic.test.ts can add an unanalyzable-sql
// finding, so the isolated length is not rules.length.

#[test]
fn not_requested_by_default() {
    assert!(!is_requested(&config("{}"), RULE).unwrap());
}

#[test]
fn production_factory_imports_do_not_match_the_test_scope() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-test-state-mutation/fixture");
    let rule_id = "postgres-no-test-state-mutation";
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: rule_id.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "testGlobs: ['tests/**']\nhelperRoots: ['helpers/**']\nimportSpecifier: '@example/db'\nexecutorFactoryNames: [openTransaction]\nreportUnmatchedExecutorNames: true",
            ).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = vec![
        root.join("tests/dynamic.test.ts"),
        root.join("src/transaction.ts"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check(&root, &config, rule_id, &files, &sources, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("`openTransaction`"));
    assert!(findings[0].message.contains("1 file(s)"));
}
