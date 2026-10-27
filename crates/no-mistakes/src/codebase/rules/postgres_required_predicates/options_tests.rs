// Output cases moved to cli_postgres_snapshot_required_predicates.
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture")
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
        relations: vec![RelationOption {
            table: "topics".into(),
            require: vec!["parent_id IS NOT NULL".into()],
            ..Default::default()
        }],
        ..Default::default()
    })
    .unwrap();
    assert!(!compiled.fail_unanalyzable);
    assert_eq!(compiled.relations[0].table, "topics");
    assert_eq!(compiled.schema.sql_include, ["migrations/**/*.sql"]);
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
    assert!(
        compile_options(&Options {
            unanalyzable_sql: "fail".into(),
            ..Default::default()
        })
        .unwrap()
        .fail_unanalyzable
    );
}

#[test]
fn dynamic_unparseable_and_unrelated_tables() {
    let relations = "sqlInclude: [\"sql/**/*.sql\"]\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]";
    let unrelated = compile_options(&Options {
        relations: vec![RelationOption {
            table: "accounts".into(),
            require: vec!["id IS NOT NULL".into()],
            ..Default::default()
        }],
        ..Default::default()
    })
    .unwrap();
    let root = fixture("fail");
    let sql = root.join("sql/001.sql");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&sql));
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        std::slice::from_ref(&sql),
        std::sync::Arc::clone(&sources),
        &config_yaml(relations),
        &[RULE_ID],
    )
    .unwrap();
    let findings = scan::scan(
        &root,
        &unrelated,
        std::slice::from_ref(&sql),
        &sources,
        Some(&facts),
    )
    .unwrap();
    let findings = findings.finish();
    assert!(findings.is_empty(), "{findings:?}");
}
