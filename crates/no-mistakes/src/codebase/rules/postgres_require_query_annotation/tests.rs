// Output cases moved to cli_postgres_snapshot_require_query_annotation.
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

#[test]
fn var_sql_retains_function_scope_and_lexical_bindings_stay_local() {
    let root = fixture("var-scopes");
    let file = ts_file(&root);
    let source = std::fs::read_to_string(&file).unwrap();
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
mod helper_tracing;
