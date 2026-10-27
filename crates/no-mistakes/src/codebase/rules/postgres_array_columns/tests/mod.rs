mod config;

use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

#[test]
fn missing_catalog_file_is_an_error() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/array-columns/pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("schemaCatalogPath: missing.json").unwrap(),
        ..RuleDef::default()
    });
    let error = super::check_with_files(&root, &config, &[root.join("schema.json")]).unwrap_err();
    assert!(error.to_string().contains("missing.json"), "{error}");
}
