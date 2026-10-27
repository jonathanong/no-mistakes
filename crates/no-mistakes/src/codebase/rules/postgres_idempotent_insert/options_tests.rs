use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-idempotent-insert/fixture")
            .join(name),
    )
}

fn config_yaml(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn include_exclude_and_option_overrides() {
    let root = fixture("fail");
    let sql = root.join("sql/001.sql");
    let error = check_with_files(&root, &config_yaml("exclude: ['[']"), &[sql]).expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
    let compiled = compile_options(&Options {
        sql_include: vec!["migrations/**/*.sql".into()],
        import_specifier: "@other/db".into(),
        executor_names: vec!["run".into()],
        unanalyzable_sql: "ignore".into(),
        scan_embedded: false,
        check_convergence: false,
        check_volatility: false,
        check_arbiter: false,
        check_triggers: false,
        check_generated: false,
        replay_safe_trigger_functions: vec!["audit".into()],
        trigger_written_columns: [("audit".into(), vec!["id".into()])].into(),
        ..Default::default()
    })
    .unwrap();
    assert!(!compiled.fail_unanalyzable);
    assert!(!compiled.scan_embedded);
    assert!(!compiled.check_convergence);
    assert!(!compiled.check_volatility);
    assert!(!compiled.check_arbiter);
    assert!(!compiled.check_triggers);
    assert!(!compiled.check_generated);
    let only_volatility = compile_options(&Options {
        check_volatility: false,
        ..Default::default()
    })
    .unwrap();
    assert!(only_volatility.check_convergence);
    assert!(!only_volatility.check_volatility);
    assert!(only_volatility.check_arbiter);
    assert!(only_volatility.check_triggers);
    assert!(only_volatility.check_generated);
    assert_eq!(compiled.schema.sql_include, ["migrations/**/*.sql"]);
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
    assert_eq!(compiled.replay_safe, ["audit"]);
    assert!(compiled
        .trigger_writes
        .iter()
        .any(|(name, columns)| name == "audit" && columns == &["id".to_string()]));
    assert!(
        compile_options(&Options {
            unanalyzable_sql: "fail".into(),
            ..Default::default()
        })
        .unwrap()
        .fail_unanalyzable
    );
}
