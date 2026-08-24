use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-add-column/fixture")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("sqlInclude: [\"migrations/**/*.sql\"]").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn config_with_allowed_migration(default: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                "sqlInclude: [\"migrations/**/*.sql\"]\nallowedMigrations:\n  - path: migrations/001.sql\n    table: posts\n    column: status\n    type: TEXT\n    nullable: false\n    default: \"'{default}'\""
            ))
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path) -> Vec<RuleFinding> {
    let files = vec![root.join("migrations/001.sql")];
    check_with_files(root, &config(), &files).unwrap()
}

fn run_with_config(root: &Path, config: NoMistakesConfig) -> Vec<RuleFinding> {
    let files = vec![root.join("migrations/001.sql")];
    check_with_files(root, &config, &files).unwrap()
}

#[test]
fn flags_alter_table_add_column() {
    let findings = run(&fixture("fail"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("posts.status"));
    assert!(findings[0].message.contains("ADD COLUMN"), "{findings:?}");
}

#[test]
fn flags_add_column_inside_do_block() {
    let findings = run(&fixture("fail-do-block"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("posts.status"));
}

#[test]
fn create_table_and_add_constraint_are_clean() {
    assert!(run(&fixture("pass")).is_empty());
}

#[test]
fn option_defaults_use_schema_sql_include() {
    let compiled = compile_options(&Options::default());
    assert_eq!(
        compiled.schema.sql_include,
        crate::codebase::postgres::PostgresSchemaOptions::default().sql_include
    );
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("migrations/does-not-exist.sql");
    let error = check_with_files(&root, &config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect PostgreSQL"),
        "{error}"
    );
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("migrations/disabled.sql");
    let mut findings = check_with_files(&root, &config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn allows_exact_configured_migration_add_column() {
    let root = fixture("allowed");
    assert!(run_with_config(&root, config_with_allowed_migration("draft")).is_empty());
}

#[test]
fn allows_exact_configured_migration_add_column_inside_static_do_block() {
    let root = fixture("allowed-do");
    assert!(run_with_config(&root, config_with_allowed_migration("draft")).is_empty());
}

#[test]
fn reports_unexpected_and_stale_allowed_migrations_bidirectionally() {
    let root = fixture("mismatch");
    let findings = run_with_config(&root, config_with_allowed_migration("published"));
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings.iter().any(|finding| {
        finding.target.as_deref() == Some("posts.status")
            && finding
                .message
                .contains("does not match an allowedMigrations entry")
    }));
    assert!(
        findings.iter().any(|finding| {
            finding.target.as_deref()
                == Some("migrations/001.sql:posts:status:TEXT:false:'published'")
                && finding
                    .message
                    .contains("stale postgres-no-add-column allowedMigrations entry")
        }),
        "{findings:#?}"
    );
}

#[test]
fn reports_allowlist_entry_stale_when_dynamic_sql_is_not_analyzed() {
    let root = fixture("dynamic");
    let findings = run_with_config(&root, config_with_allowed_migration("draft"));
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0]
        .message
        .contains("stale postgres-no-add-column allowedMigrations entry"));
}

#[test]
fn reports_duplicate_allowed_migrations() {
    let root = fixture("allowed");
    let mut config = config_with_allowed_migration("draft");
    let duplicate = config.rules[0].options["allowedMigrations"][0].clone();
    config.rules[0].options["allowedMigrations"]
        .as_sequence_mut()
        .unwrap()
        .push(duplicate);
    let findings = run_with_config(&root, config);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0]
        .message
        .contains("duplicate postgres-no-add-column allowedMigrations entry"));
}

#[test]
fn duplicate_allowed_migration_does_not_also_report_as_stale() {
    let root = fixture("allowed");
    let mut config = config_with_allowed_migration("draft");
    let duplicate = config.rules[0].options["allowedMigrations"][0].clone();
    config.rules[0].options["allowedMigrations"]
        .as_sequence_mut()
        .unwrap()
        .push(duplicate);

    let findings = run_with_config(&root, config);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0]
        .message
        .contains("duplicate postgres-no-add-column allowedMigrations entry"));
}
