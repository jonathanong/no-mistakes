mod cast_context;
mod directional_keysets;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

mod bounded_iteration;

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("sqlInclude: [\"sql/**/*.sql\"]\nexecutorNames: []")
                .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
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
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("sql/disabled.sql");
    let mut findings = check_with_files(&root, &config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn compile_options_default_to_exists_set_op() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert!(compiled.fail_unanalyzable);
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
fn rejects_unknown_options() {
    let error = compile_options(&Options {
        unanalyzable_sql: "fial".into(),
        ..Default::default()
    })
    .err()
    .expect("mode");
    assert!(error.to_string().contains("unanalyzableSql"), "{error}");
    let error = compile_options(&Options {
        banned_shapes: vec!["other-shape".into()],
        ..Default::default()
    })
    .err()
    .expect("shape");
    assert!(error.to_string().contains("bannedShapes"), "{error}");
    assert!(
        !compile_options(&Options {
            banned_shapes: vec!["correlated-exists-set-operation".into()],
            unanalyzable_sql: "ignore".into(),
            ..Default::default()
        })
        .unwrap()
        .fail_unanalyzable
    );
}

#[test]
fn include_exclude_and_option_overrides() {
    let root = fixture("fail");
    let sql = root.join("sql/001.sql");
    let error = check_with_files(
        &root,
        &config_yaml("include: ['[']"),
        std::slice::from_ref(&sql),
    )
    .expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
    let error = check_with_files(
        &root,
        &config_yaml("exclude: ['[']"),
        std::slice::from_ref(&sql),
    )
    .expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
    let compiled = compile_options(&Options {
        sql_include: vec!["migrations/**/*.sql".into()],
        import_specifier: "@other/db".into(),
        executor_names: vec!["run".into()],
        unanalyzable_sql: "ignore".into(),
        ..Default::default()
    })
    .unwrap();
    assert!(!compiled.fail_unanalyzable);
    assert_eq!(compiled.schema.sql_include, ["migrations/**/*.sql"]);
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
}

#[test]
fn flags_correlated_exists_unions_recovered_from_sql_builders() {
    let root = fixture("fail-embedded-builders");
    let builders = root.join("src/builders.ts");
    let source = std::fs::read_to_string(&builders).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &builders,
        &source,
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(embedded.fragments.iter().any(|fragment| {
        fragment.sql_text.as_deref().is_some_and(|sql| {
            sql.contains("sql_dynamic_outer.column") && sql.contains("UNION ALL")
        })
    }));
}

#[test]
fn reports_an_unsafe_fluent_builder_once() {
    let root = fixture("fail-embedded-builder-overlap");
    let builders = root.join("src/builders.ts");
    let source = std::fs::read_to_string(&builders).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &builders,
        &source,
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(embedded.fragments.len(), 1, "{:#?}", embedded.fragments);
}

#[test]
fn reports_a_multiline_builder_overlap_once() {
    let root = fixture("fail-embedded-builder-multiline-overlap");
    let builders = root.join("src/builders.ts");
    let source = std::fs::read_to_string(&builders).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &builders,
        &source,
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(embedded.fragments.len(), 1, "{:#?}", embedded.fragments);
}

#[test]
fn duplicate_statement_files_dedupe_identical_findings() {
    let root = fixture("fail");
    let sql = root.join("sql/001.sql");
    let findings = check_with_files(&root, &config(), &[sql.clone(), sql]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
}

mod predicate_escaping;

#[cfg(test)]
mod negated_keysets;
mod semantic_zero;

#[path = "plain_return_identity_tests.rs"]
mod plain_return_identity;
