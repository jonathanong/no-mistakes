//! Fixture-backed checks for how `ORDER BY` keys are matched to arbiter expressions.
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn kinds(scenario: &str) -> Vec<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/conflict-ordering/cases")
        .join(scenario);
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(
            "schemaCatalogPath: schema.json",
        ),
        ..Default::default()
    });
    let files = [root.join("src/insert.ts"), root.join("schema.json")];
    check_with_files(&root, &config, &files)
        .unwrap()
        .into_iter()
        .map(|finding| finding.target.unwrap_or_default())
        .collect()
}

#[test]
fn a_bare_order_by_column_matches_its_qualified_select_expression() {
    assert!(kinds("pass-bare-order-by").is_empty());
}

#[test]
fn an_ambiguous_or_wrongly_ordered_bare_column_is_still_reported() {
    // Ambiguous (two relations declare the name), unknown (plain table), reversed order.
    assert_eq!(kinds("fail-bare-order-by"), ["noncanonical-order"; 3]);
}

#[test]
fn an_arbiter_column_missing_from_the_insert_list_is_a_constant_key() {
    assert!(kinds("pass-default-arbiter").is_empty());
}

#[test]
fn an_omitted_arbiter_column_does_not_excuse_missing_order_or_column_list() {
    assert_eq!(
        kinds("fail-default-arbiter"),
        ["missing-canonical-order", "unresolved-source-order"]
    );
}

#[test]
fn a_recovered_template_placeholder_is_a_bound_parameter_key() {
    assert!(kinds("pass-template-constant-key").is_empty());
}

#[test]
fn only_recovered_placeholder_positions_count_as_constant_keys() {
    // The control stays unordered; user-spelled markers are columns, even next to a real one.
    assert_eq!(
        kinds("fail-template-constant-key"),
        ["missing-canonical-order"; 3]
    );
}
