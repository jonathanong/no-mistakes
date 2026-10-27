use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

// Output cases moved to cli_postgres_snapshot_sql_statement_policy.

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-statement-policy/fixture")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("sqlInclude: [\"sql/**/*.sql\"]").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn option_defaults_use_schema_sql_include_and_supported_kinds() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert_eq!(
        compiled.schema.sql_include,
        crate::codebase::postgres::PostgresSchemaOptions::default().sql_include
    );
    for kind in DEFAULT_BANNED {
        assert!(compiled.banned.contains(*kind), "{kind}");
    }
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("sql/does-not-exist.sql");
    let error = check_with_files(&root, &config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect PostgreSQL"),
        "{error}"
    );
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("sql/disabled.sql");
    let mut findings = check_with_files(&root, &config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

fn embedded_config(unanalyzable: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(), scope: Some(RuleScope::Repository),
            include: vec!["**/*.test.ts".into(), "sql/**/*.sql".into()],
            exclude: vec!["excluded.test.ts".into()],
            options: serde_yaml::from_str(&format!(
                "importSpecifier: '@example/db'\ntrustedSqlTags: [{{module: '@example/db', name: sql}}]\nexecutorFactoryNames: [beginTransaction]\nexecutorTypeNames: [TransactionQuery]\nsqlInclude: ['sql/seeds/*.sql']\nunanalyzableSql: {unanalyzable}"
            )).unwrap(), ..Default::default()
        }], ..Default::default()
    }
}

#[test]
fn dynamic_and_unparseable_sql_follow_policy() {
    // Finding cases moved to cli_postgres_snapshot_sql_statement_policy.
    let root = fixture("embedded");
    let files = [root.join("dynamic.test.ts")];
    assert!(check_with_files(&root, &embedded_config("invalid"), &files)
        .unwrap_err()
        .to_string()
        .contains("unanalyzableSql"));
}

#[test]
fn embedded_suppression_uses_ts_source_lines() {
    let root = fixture("embedded");
    let files = [
        root.join("suppressed.test.ts"),
        root.join("disabled.test.ts"),
    ];
    let mut findings = check_with_files(&root, &embedded_config("fail"), &files).unwrap();
    assert_eq!(findings.len(), 3);
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn legacy_sql_only_policy_does_not_read_or_parse_ts() {
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    crate::codebase::postgres::configure_prepared_postgres_plan(
        std::path::Path::new("/repo"),
        &config(),
        &mut plan,
    )
    .unwrap();
    assert!(!plan.postgres_dml);
    assert!(
        crate::codebase::postgres::configured_embedded_sql_options_for_checks(&config())
            .unwrap()
            .is_empty()
    );
    let root = fixture("embedded");
    let files = [root.join("unparsed.ts")];
    let sources = super::super::source_store_for_files(&files);
    // Emptiness of the sql-only scan is in the snapshot. This call must not read TS.
    check_with_files_and_sources(&root, &config(), &files, &sources).unwrap();
    assert_eq!(sources.physical_read_count(), 0);
}
