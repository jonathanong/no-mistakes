use super::{tests::fixture, *};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

// Output cases moved to cli_postgres_snapshot_sql_statement_policy.

fn config(settings: bool) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef { rule: RULE_ID.into(), scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(if settings {
                "bannedStatements: [ddl]\nbannedSettings: [session_replication_role, app.guard]\nimportSpecifier: '@example/db'\nunanalyzableSql: ignore"
            } else { "bannedStatements: [ddl]\nsqlInclude: ['**/*.sql']" }).unwrap(), ..Default::default() }],
        ..Default::default()
    }
}

#[test]
fn ddl_expands_exactly_supported_kinds_and_unknown_values_error() {
    let compiled = compile_options(&Options {
        banned_statements: vec!["DdL".into()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.banned.len(), 29);
    for kind in crate::codebase::postgres::SUPPORTED_KINDS {
        assert!(compiled.banned.contains(*kind));
    }
    for invalid in ["dml", "CREATE FOO", ""] {
        let error = compile_options(&Options {
            banned_statements: vec![invalid.into()],
            ..Default::default()
        })
        .err()
        .unwrap();
        assert!(
            error.to_string().contains("unknown bannedStatements"),
            "{error}"
        );
    }
    let normalized = compile_options(&Options {
        banned_statements: vec![" create   database ".into()],
        ..Default::default()
    })
    .unwrap();
    assert!(normalized.banned.contains("CREATE DATABASE"));
}

#[test]
fn new_findings_honor_existing_sql_suppression() {
    let root = fixture("expanded");
    let files = [root.join("disabled.sql")];
    let mut findings = check_with_files(&root, &config(true), &files).unwrap();
    assert_eq!(findings.len(), 2);
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}
