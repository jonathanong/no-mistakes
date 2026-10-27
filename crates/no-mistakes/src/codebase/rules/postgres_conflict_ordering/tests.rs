mod patch_coverage;
mod variants;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

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

include!("tests/preparation_and_unknown.rs");
