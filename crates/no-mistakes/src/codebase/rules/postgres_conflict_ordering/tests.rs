mod patch_coverage;
mod variants;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/conflict-ordering")
}

pub(super) fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("cases").join(scenario)
}

pub(super) fn config() -> NoMistakesConfig {
    config_with_options("schemaCatalogPath: schema.json")
}

fn config_with_options(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(options),
        ..Default::default()
    });
    config
}

fn files(root: &Path) -> Vec<PathBuf> {
    vec![root.join("src/insert.ts"), root.join("schema.json")]
}

fn findings(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    check_with_files(&root, &config(), &files(&root)).unwrap()
}

#[test]
fn rejects_a_multi_row_source_without_canonical_order() {
    let findings = findings("fail-missing-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("missing-canonical-order")
    );
    assert!(findings[0].message.contains("catalog arbiter"));
}

#[test]
fn rejects_a_noncanonical_source_order() {
    let findings = findings("fail-noncanonical-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("noncanonical-order"));
}

#[test]
fn rejects_a_source_that_cannot_be_mapped_to_target_columns() {
    let findings = findings("fail-unresolved-source-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("unresolved-source-order")
    );
}

#[test]
fn accepts_the_catalog_key_prefix() {
    assert!(findings("pass-canonical-order").is_empty());
}

#[test]
fn resolves_top_level_select_aliases_in_the_source_order() {
    let root = fixture("pass-canonical-order");
    let findings = check_with_files(
        &root,
        &config(),
        &[root.join("src/alias.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn follows_the_shared_typed_transaction_executor_facts() {
    let root = fixture("pass-canonical-order");
    let findings = check_with_files(
        &root,
        &config(),
        &[root.join("src/transaction.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn rejects_targetless_multi_row_do_nothing() {
    let findings = findings("fail-targetless");
    assert_eq!(findings[0].target.as_deref(), Some("targetless-arbiter"));
}

#[test]
fn accepts_expression_index_order_with_a_terminal_tie_breaker() {
    let findings = findings("pass-expression");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn rejects_differently_ordered_inferred_indexes() {
    let findings = findings("fail-ambiguous");
    assert_eq!(findings[0].target.as_deref(), Some("ambiguous-arbiter"));
}

#[test]
fn rejects_a_conflict_target_whose_written_order_differs_from_the_catalog() {
    let findings = findings("fail-reversed-target");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("noncanonical-target"));
}

#[test]
fn resolves_a_partial_unique_index_by_its_inference_predicate() {
    assert!(findings("pass-partial").is_empty());
    let findings = findings("fail-partial-predicate");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unresolved-arbiter"));
}

#[test]
fn resolves_a_named_unique_constraint_to_its_catalog_key_order() {
    let findings = findings("pass-constraint");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn scans_opted_in_static_sql_sources() {
    let root = fixture("pass-sql-include");
    let findings = check_with_files(
        &root,
        &config_with_options(
            "schemaCatalogPath: schema.json\ninclude: ['src/**/*.ts']\nsqlInclude: ['queries/**/*.sql']",
        ),
        &[root.join("queries/insert.sql"), root.join("schema.json")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn scopes_sql_file_safe_directives_to_their_own_statement() {
    let root = fixture("fail-sql-statement-directive");
    let findings = check_with_files(
        &root,
        &config_with_options(
            "schemaCatalogPath: schema.json\ninclude: ['src/**/*.ts']\nsqlInclude: ['queries/**/*.sql']",
        ),
        &[root.join("queries/inserts.sql"), root.join("schema.json")],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("missing-canonical-order")
    );
    assert_eq!(findings[0].line, 5);
}

include!("tests/preparation_and_unknown.rs");
include!("tests/append_and_napi.rs");
