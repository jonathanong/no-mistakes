use super::{check_with_files, RULE_ID};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

mod behavior;
mod config;
mod entrypoint;
mod support;

#[test]
fn rule_id_is_stable() {
    assert_eq!(RULE_ID, "postgres-key-column-types");
}

#[test]
fn check_with_files_reads_a_prepared_catalog_deterministically() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        options: serde_yaml::from_str("schemaCatalogPath: mixed.json\nallowedTypes: [uuid]\n")
            .unwrap(),
        ..Default::default()
    });
    let files = vec![root.join("mixed.json")];
    let first = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(first, check_with_files(&root, &config, &files).unwrap());
    assert!(!first.is_empty());
}

#[test]
fn rule_path_filter_can_skip_a_configured_catalog() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        include: vec!["migrations/**".to_string()],
        options: serde_yaml::from_str("schemaCatalogPath: mixed.json\nallowedTypes: [uuid]\n")
            .unwrap(),
        ..Default::default()
    });
    let files = vec![root.join("mixed.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = super::check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert!(
        findings.is_empty(),
        "scope-excluded catalog should not be checked: {findings:#?}"
    );
}

#[test]
fn catalog_scan_errors_propagate_through_the_rule_entrypoint() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        options: serde_yaml::from_str(
            "schemaCatalogPath: missing-primary-index.json\nallowedTypes: [uuid]\n",
        )
        .unwrap(),
        ..Default::default()
    });
    let files = vec![root.join("missing-primary-index.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let error = super::check_with_files_and_sources(&root, &config, &files, &sources).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("has a primary key but no primary index name"),
        "{error:#}"
    );
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/key-column-types/catalogs")
}
