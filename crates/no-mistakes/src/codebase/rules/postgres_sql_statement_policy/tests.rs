use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
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

fn run(root: &Path) -> Vec<RuleFinding> {
    let files = vec![root.join("sql/001.sql")];
    check_with_files(root, &config(), &files).unwrap()
}

#[test]
fn flags_create_table() {
    let findings = run(&fixture("fail"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("CREATE TABLE"));
    assert!(findings[0].message.contains("CREATE TABLE"), "{findings:?}");
}

#[test]
fn flags_default_banned_kinds() {
    let findings = run(&fixture("fail-kinds"));
    let kinds: Vec<_> = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();
    assert_eq!(
        kinds,
        [
            "CREATE TABLE",
            "ALTER TABLE",
            "CREATE INDEX",
            "CREATE VIEW",
            "TRUNCATE",
            "DROP INDEX",
            "DROP VIEW",
        ],
        "{findings:?}"
    );
}

#[test]
fn flags_create_table_inside_do_block() {
    let findings = run(&fixture("fail-do-block"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("CREATE TABLE"));
}

#[test]
fn inserts_and_functions_are_clean() {
    assert!(run(&fixture("pass")).is_empty());
}

#[test]
fn custom_banned_statements_narrow_the_rule() {
    let root = fixture("fail-kinds");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "sqlInclude: [\"sql/**/*.sql\"]\nbannedStatements: [\"TRUNCATE\"]",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = vec![root.join("sql/001.sql")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("TRUNCATE"));
}

#[test]
fn option_defaults_use_schema_sql_include_and_filaments_kinds() {
    let compiled = compile_options(&Options::default());
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
