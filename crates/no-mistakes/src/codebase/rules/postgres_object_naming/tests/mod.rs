mod config;
mod edges;
mod expand;
mod findings;
mod replacement;
mod support;
mod valid;

use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

use support::PROPOSED;

#[test]
fn rule_id_is_stable() {
    assert_eq!(super::RULE_ID, "postgres-object-naming");
}

#[test]
fn proposed_config_compiles_and_stale_allow_is_reported() {
    let messages = support::messages(PROPOSED, serde_json::json!({}));
    assert_eq!(
        messages,
        vec!["schema.json: stale postgres-object-naming allow entry: table:legacy_cfg_values"]
    );
}

#[test]
fn check_with_files_reads_a_prepared_catalog() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/object-naming/pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(
            "schemaCatalogPath: schema.json\npatterns:\n  table: '^[a-z][a-z0-9_]*$'\nplural:\n  enabled: true\n",
        )
        .unwrap(),
        ..RuleDef::default()
    });
    let findings = super::check_with_files(&root, &config, &[root.join("schema.json")]).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
    let again = super::check_with_files(&root, &config, &[root.join("schema.json")]).unwrap();
    assert_eq!(findings, again);
}

#[test]
fn custom_message_and_include_filter_the_catalog() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/object-naming/pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        message: Some("use the shared names".to_string()),
        options: serde_yaml::from_str(
            "schemaCatalogPath: schema.json\npatterns:\n  table: '^widgets$'\n",
        )
        .unwrap(),
        ..RuleDef::default()
    });
    let messages = super::check_with_files(&root, &config, &[root.join("schema.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec!["schema.json: table:order_line_items: use the shared names".to_string()]
    );
    config.rules[0].include = vec!["missing.json".to_string()];
    assert!(
        super::check_with_files(&root, &config, &[root.join("schema.json")])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn missing_catalog_file_is_an_error() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/object-naming/pass");
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
