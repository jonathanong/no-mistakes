use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation")
}

pub(super) fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("fixture").join(scenario)
}

pub(super) fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
        ..Default::default()
    });
    config
}

fn default_config() -> NoMistakesConfig {
    config_with_options("{}")
}

pub(super) fn ts_file(root: &Path) -> PathBuf {
    root.join("src/query.ts")
}

fn findings_for(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    let file = ts_file(&root);
    check_with_files(&root, &default_config(), &[file]).unwrap()
}

#[test]
fn fail_fixture_reports_missing_annotation() {
    let findings = findings_for("fail");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, "src/query.ts");
    assert!(findings[0].line > 0);
    assert_eq!(findings[0].target.as_deref(), Some("annotation"));
    assert!(findings[0].message.contains("/* name */"), "{findings:#?}");
}

#[test]
fn annotated_query_is_clean() {
    assert!(findings_for("pass-annotated").is_empty());
}

#[test]
fn chained_executor_returns_preserve_the_exact_executed_query() {
    let root = fixture("chained-executor-return");
    let file = root.join("src/query.mts");
    let source = std::fs::read_to_string(&file).unwrap();
    let findings = check_with_files(
        &root,
        &config_with_options("importSpecifier: '@example/db'\nexecutorNames: [write]"),
        &[file],
    )
    .unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// finding:").then_some(index + 1))
        .collect::<Vec<_>>();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        expected,
        "{findings:#?}"
    );
    let actual = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();
    for (finding, line) in findings.iter().zip(expected) {
        let marker = source.lines().nth(line - 1).unwrap();
        assert!(
            finding
                .message
                .contains(if marker.contains("finding: missing") {
                    "query execution must start"
                } else {
                    "leading SQL is unanalyzable"
                }),
            "{actual:#?}"
        );
    }
}

#[test]
fn var_sql_retains_function_scope_and_lexical_bindings_stay_local() {
    let root = fixture("var-scopes");
    let file = ts_file(&root);
    let source = std::fs::read_to_string(&file).unwrap();
    let findings = check_with_files(
        &root,
        // This fixture separately pins legacy SQL recovery, including opaque
        // bindings; default unknown reporting is covered by helper-tracing.
        &config_with_options("importSpecifier: '@example/db'\nunanalyzableSql: ignore"),
        std::slice::from_ref(&file),
    )
    .unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// finding:").then_some(index + 1))
        .collect::<Vec<_>>();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        expected
    );

    let facts = crate::codebase::postgres::extract_embedded_sql_from_source(
        &file,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(facts
        .calls
        .iter()
        .all(|call| call.declaration_line != Some(0)));
    for (index, line) in source.lines().enumerate() {
        if line.contains("// unknown:") || line.contains("// known:") {
            let call = facts
                .calls
                .iter()
                .find(|call| call.line as usize == index + 1)
                .unwrap();
            assert_eq!(
                call.sql_text.is_some(),
                line.contains("// known:"),
                "{line}"
            );
        }
    }
}

#[test]
fn transaction_commands_are_clean() {
    assert!(findings_for("pass-transaction").is_empty());
}

#[test]
fn line_comment_is_not_an_annotation() {
    assert_eq!(findings_for("fail-line-comment").len(), 1);
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("src/disabled.ts");
    let mut findings =
        check_with_files(&root, &default_config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

mod config;
mod helper_contexts;
mod helper_tracing;
mod helper_value;
