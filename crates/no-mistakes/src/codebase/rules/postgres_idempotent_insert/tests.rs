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

fn default_config() -> NoMistakesConfig {
    config_yaml("sqlInclude: [\"sql/**/*.sql\"]")
}

#[test]
fn compile_options_default_checks_on() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert!(compiled.check_convergence);
    assert!(compiled.check_volatility);
    assert!(compiled.check_arbiter);
    assert!(compiled.check_triggers);
    assert!(compiled.check_generated);
    assert!(compiled.scan_embedded);
    assert!(compiled.replay_safe.is_empty());
}

#[test]
fn prepared_embedded_facts_missing_errors() {
    let root = fixture("fail");
    let file = root.join("src/query.ts");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    let error = check_with_files_sources_and_facts(
        &root,
        &default_config(),
        std::slice::from_ref(&file),
        &sources,
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .expect_err("embedded facts");
    assert!(error.to_string().contains("prepared"), "{error}");
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("sql/does-not-exist.sql");
    for policy in ["report", "ignore"] {
        let config = config_yaml(&format!(
            "sqlInclude: ['sql/**/*.sql']\nunanalyzable: {policy}"
        ));
        // SQL recovery policy does not turn a standalone source read error into clean input.
        let error =
            check_with_files(&root, &config, std::slice::from_ref(&missing)).expect_err("read");
        assert!(
            error.to_string().contains("failed to collect PostgreSQL"),
            "{error}"
        );
    }
}

#[test]
fn invalid_include_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_yaml("include: ['[']"),
        &[root.join("sql/001.sql")],
    )
    .expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn rejects_unknown_unanalyzable_sql() {
    let error = compile_options(&Options {
        unanalyzable_sql: "fial".into(),
        ..Default::default()
    })
    .err()
    .expect("mode");
    assert!(error.to_string().contains("unanalyzableSql"), "{error}");
}

#[test]
fn embedded_queries_build_one_trigger_timeline_on_both_fact_paths() {
    for state in ["live", "dropped", "replaced"] {
        let root = fixture(&format!("embedded-trigger-final-{state}"));
        let files = [
            root.join("app/q.ts"),
            root.join("lib/q.ts"),
            root.join("sql/001.sql"),
        ];
        let config = config_yaml(
            "sqlInclude: ['sql/**/*.sql']\nimportSpecifier: '@example/db'\nexecutorNames: [query]",
        );
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let findings = crate::diagnostics::with_observer(Some(observer.clone()), || {
            check_with_files(&root, &config, &files).unwrap()
        });
        assert_eq!(
            observer.snapshot().work["postgres.trigger.timeline_builds"],
            1
        );
        let sources = super::super::source_store_for_files(&files);
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let unprepared = crate::diagnostics::with_observer(Some(observer.clone()), || {
            check_with_files_and_sources(&root, &config, &files, &sources).unwrap()
        });
        assert_eq!(
            observer.snapshot().work["postgres.trigger.timeline_builds"],
            1
        );
        assert_eq!(unprepared, findings, "{state}");
    }
}

#[test]
fn drop_position_ignores_comments_strings_and_quoted_keyword_names() {
    let root = fixture("pass-trigger-drop-keywords-in-comments");
    let sql = std::fs::read_to_string(root.join("sql/001.sql")).unwrap();
    let facts = crate::codebase::postgres::statements::extract_sql_statement_facts(&sql);
    assert_eq!(facts.triggers[0].line, 1);
    assert_eq!(facts.triggers[1].line, 4);
}
