use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

const RULE: &str = "postgres-lock-ordering";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-ordering/fixture/fail-unmatched-executor-names")
}

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

fn run(yaml: &str) -> Vec<RuleFinding> {
    let root = root();
    let files = vec![root.join("src/lock.ts")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    check(&root, &config(yaml), RULE, &files, &sources, None).unwrap()
}

#[test]
fn not_requested_by_default() {
    assert!(!is_requested(&config("{}"), RULE).unwrap());
    assert!(run("{ importSpecifier: '@example/db', executorFactoryNames: [nope] }").is_empty());
}

#[test]
fn standalone_preparation_reports_only_unmatched_names() {
    let findings = run(
        "{ importSpecifier: '@example/db', reportUnmatchedExecutorNames: true, \
         executorFactoryNames: [openTransaction, openTransacton], \
         executorTypeNames: [TransactionQuery] }",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("`openTransacton`"));
    assert!(findings[0].message.contains("1 file(s)"));
    assert!(
        findings[0]
            .message
            .contains("a relative import matches only when it resolves into that package"),
        "{findings:?}"
    );
}

#[test]
fn excluded_files_cannot_match_so_every_name_is_reported() {
    let findings = run(
        "{ importSpecifier: '@example/db', reportUnmatchedExecutorNames: true, \
         exclude: ['src/**'], executorFactoryNames: [openTransaction], \
         executorTypeNames: [TransactionQuery] }",
    );
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings[0].message.contains("0 file(s)"));
}

#[test]
fn an_empty_specifier_names_any_module() {
    let findings = run(
        "{ importSpecifier: '', reportUnmatchedExecutorNames: true, executorFactoryNames: [nope], \
         executorNames: [query] }",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("any module"), "{findings:?}");
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
