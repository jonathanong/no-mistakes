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
fn embedded_executors_honor_application_scope_and_sql_include() {
    let root = fixture("embedded");
    let files = [
        "policy.test.ts",
        "excluded.test.ts",
        "runtime.ts",
        "sql/migration.sql",
    ]
    .map(|file| root.join(file));
    let findings = check_with_files(&root, &embedded_config("ignore"), &files).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.file.as_str(), f.line, f.target.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("policy.test.ts", 2, Some("ALTER TABLE")),
            ("policy.test.ts", 4, Some("CREATE VIEW")),
            ("policy.test.ts", 6, Some("TRUNCATE"))
        ]
    );
}

#[test]
fn dynamic_and_unparseable_sql_follow_policy() {
    let root = fixture("embedded");
    let files = [root.join("dynamic.test.ts")];
    assert!(check_with_files(&root, &embedded_config("ignore"), &files)
        .unwrap()
        .is_empty());
    let findings = check_with_files(&root, &embedded_config("fail"), &files).unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert!(findings
        .iter()
        .all(|f| f.target.as_deref() == Some("unanalyzable-sql")));
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
fn embedded_static_routine_ddl_reuses_schema_categories() {
    let root = fixture("embedded");
    let findings = check_with_files(
        &root,
        &embedded_config("ignore"),
        &[root.join("routines.test.ts")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.line, f.target.as_deref()))
            .collect::<Vec<_>>(),
        vec![(4, Some("CREATE TABLE")), (5, Some("ALTER TABLE"))]
    );
}

#[test]
fn legacy_sql_only_policy_does_not_read_or_parse_ts() {
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    crate::codebase::postgres::configure_prepared_postgres_plan(&config(), &mut plan).unwrap();
    assert!(!plan.postgres_dml);
    assert!(
        crate::codebase::postgres::configured_embedded_sql_options_for_checks(&config())
            .unwrap()
            .is_empty()
    );
    let root = fixture("embedded");
    let files = [root.join("unparsed.ts")];
    let sources = super::super::source_store_for_files(&files);
    assert!(
        check_with_files_and_sources(&root, &config(), &files, &sources)
            .unwrap()
            .is_empty()
    );
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn custom_banned_statements_also_narrow_embedded_calls() {
    let root = fixture("embedded");
    let mut config = embedded_config("fail");
    config.rules[0].options["bannedStatements"] = serde_yaml::from_str("[TRUNCATE]").unwrap();
    let findings = check_with_files(&root, &config, &[root.join("policy.test.ts")]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert_eq!(findings[0].target.as_deref(), Some("TRUNCATE"));
}
