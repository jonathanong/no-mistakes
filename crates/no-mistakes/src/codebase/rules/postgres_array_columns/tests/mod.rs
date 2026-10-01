mod config;
mod examples;
mod support;

use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

#[test]
fn rule_id_is_stable() {
    assert_eq!(super::RULE_ID, "postgres-array-columns");
}

#[test]
fn check_with_files_reads_a_prepared_catalog() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/array-columns/pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("schemaCatalogPath: schema.json").unwrap(),
        ..RuleDef::default()
    });
    let files = vec![root.join("schema.json")];
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
    assert_eq!(
        findings,
        super::check_with_files(&root, &config, &files).unwrap()
    );
}

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
