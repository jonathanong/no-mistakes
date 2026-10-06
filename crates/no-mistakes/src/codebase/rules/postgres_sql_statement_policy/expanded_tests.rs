use super::{tests::fixture, *};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

pub(super) fn config(settings: bool) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef { rule: RULE_ID.into(), scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(if settings {
                "bannedStatements: [ddl]\nbannedSettings: [session_replication_role, app.guard]\nimportSpecifier: '@example/db'\nunanalyzableSql: ignore"
            } else { "bannedStatements: [ddl]\nsqlInclude: ['**/*.sql']" }).unwrap(), ..Default::default() }],
        ..Default::default()
    }
}

#[test]
fn every_new_kind_and_pg_variants_are_reported() {
    let root = fixture("expanded");
    let findings = check_with_files(&root, &config(false), &[root.join("kinds.sql")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| f.target.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "CREATE DATABASE",
            "DROP DATABASE",
            "ALTER DATABASE",
            "ALTER SYSTEM",
            "CREATE SCHEMA",
            "ALTER SCHEMA",
            "DROP SCHEMA",
            "CREATE TRIGGER",
            "DROP TRIGGER",
            "CREATE FUNCTION",
            "CREATE PROCEDURE",
            "DROP FUNCTION",
            "DROP PROCEDURE",
            "DROP TABLE",
            "CREATE TYPE",
            "DROP TYPE",
            "CREATE TABLE",
            "ALTER TABLE",
            "CREATE TRIGGER",
        ]
    );
}

#[test]
fn setting_commands_are_named_case_insensitively_and_literals_are_inert() {
    let root = fixture("expanded");
    let findings = check_with_files(&root, &config(true), &[root.join("settings.sql")]).unwrap();
    let settings: Vec<_> = findings
        .iter()
        .filter(|f| {
            f.target
                .as_deref()
                .is_some_and(|target| target.starts_with("setting:"))
        })
        .map(|f| f.line)
        .collect();
    assert_eq!(settings, (1..=9).collect::<Vec<_>>());
}

#[test]
fn routines_and_static_execute_share_the_new_policy_facts() {
    let root = fixture("expanded");
    let findings = check_with_files(&root, &config(true), &[root.join("routines.sql")]).unwrap();
    let observed: Vec<_> = findings
        .iter()
        .map(|f| (f.line, f.target.as_deref().unwrap()))
        .collect();
    assert_eq!(
        observed,
        vec![
            (2, "ALTER DATABASE"),
            (3, "setting:session_replication_role"),
            (4, "setting:session_replication_role"),
            (5, "CREATE DATABASE"),
            (7, "CREATE FUNCTION"),
            (8, "CREATE SCHEMA"),
            (9, "setting:session_replication_role"),
            (10, "DROP SCHEMA"),
            (12, "CREATE PROCEDURE"),
            (13, "ALTER SYSTEM"),
            (14, "DROP DATABASE"),
        ]
    );
}

#[test]
fn embedded_utility_grammar_gaps_still_report_banned_kinds_and_settings() {
    let root = fixture("expanded");
    let findings =
        check_with_files(&root, &config(true), &[root.join("embedded.test.ts")]).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.line, f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        vec![
            (2, "CREATE DATABASE"),
            (3, "setting:session_replication_role"),
            (4, "setting:session_replication_role"),
            (5, "ALTER SYSTEM"),
            (6, "CREATE PROCEDURE"),
        ]
    );
}

#[test]
fn ddl_expands_exactly_supported_kinds_and_unknown_values_error() {
    let compiled = compile_options(&Options {
        banned_statements: vec!["DdL".into()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.banned.len(), 23);
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
