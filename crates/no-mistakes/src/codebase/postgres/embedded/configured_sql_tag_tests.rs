use super::{extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions, TrustedSqlTag};
use crate::codebase::rules::postgres_no_offset::{self, check_with_files};
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn options() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[TrustedSqlTag {
        module: "@example/db".into(),
        name: "sql".into(),
    }])
}

fn calls(name: &str) -> Vec<super::EmbeddedSqlCall> {
    let path = fixture(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    extract_embedded_sql_from_source(&path, &source, &options()).calls
}

fn config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: postgres_no_offset::RULE_ID.into(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(
            "importSpecifier: '@example/db'\ntrustedSqlTags:\n  - {module: '@example/db', name: sql}\nunanalyzableSql: fail\n",
        ),
        ..Default::default()
    });
    config
}

fn finding_lines(name: &str) -> Vec<(usize, Option<String>)> {
    let path = fixture(name);
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    check_with_files(root, &config(), std::slice::from_ref(&path))
        .unwrap_or_else(|error| panic!("{name}: {error}"))
        .into_iter()
        .map(|finding| (finding.line, finding.target))
        .collect()
}

#[test]
fn import_from_the_configured_module_is_trusted() {
    let calls = calls("configured-sql-tag-trusted.ts");
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("\n    SELECT id\n    FROM documents\n    WHERE account_id = sql_placeholder_1\n  ")
    );
    assert!(
        finding_lines("configured-sql-tag-trusted.ts").is_empty(),
        "a configured tag is analyzable under unanalyzableSql: fail"
    );
}

#[test]
fn local_sql_helper_stays_unanalyzable() {
    let calls = calls("configured-sql-tag-local-helper.ts");
    assert_eq!(calls.len(), 1, "the call must stay a candidate: {calls:?}");
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        finding_lines("configured-sql-tag-local-helper.ts"),
        [(7, Some("unanalyzable".into()))]
    );
}

#[test]
fn sql_imported_from_an_unconfigured_package_stays_unanalyzable() {
    let calls = calls("configured-sql-tag-unconfigured.ts");
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        finding_lines("configured-sql-tag-unconfigured.ts"),
        [(5, Some("unanalyzable".into()))]
    );
}

#[test]
fn shadowed_sql_parameter_is_not_the_configured_binding() {
    let calls = calls("configured-sql-tag-shadowed-param.ts");
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert!(calls[0]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("sql_placeholder_1"));
    assert_eq!(calls[1].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        finding_lines("configured-sql-tag-shadowed-param.ts"),
        [(13, Some("unanalyzable".into()))]
    );
}

#[test]
fn cased_sql_export_from_sql_template_strings_stays_trusted() {
    let calls = calls("configured-sql-tag-cased-export.ts");
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert!(calls[0]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("sql_placeholder_1"));
    assert!(finding_lines("configured-sql-tag-cased-export.ts").is_empty());
}

#[test]
fn renamed_sql_export_from_sql_template_strings_stays_unanalyzable() {
    let calls = calls("configured-sql-tag-renamed-export.ts");
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        finding_lines("configured-sql-tag-renamed-export.ts"),
        [(5, Some("unanalyzable".into()))]
    );
}
