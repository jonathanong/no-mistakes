mod config;
mod examples;
mod pin;
mod support;

use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

#[test]
fn rule_id_is_stable() {
    assert_eq!(super::RULE_ID, "postgres-finite-text-columns");
}

#[test]
fn check_with_files_reads_a_prepared_catalog() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/finite-text/pass");
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
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/finite-text/pass");
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

#[test]
fn custom_message_and_include_filter_apply_to_catalog_path() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/finite-text/review-followups");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        message: Some("replace this with a lookup table".to_string()),
        options: serde_yaml::from_str("schemaCatalogPath: schema.json").unwrap(),
        ..RuleDef::default()
    });
    let files = vec![root.join("schema.json")];
    let messages = super::check_with_files(&root, &config, &files)
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert!(!messages.is_empty());
    assert!(
        messages
            .iter()
            .all(|message| message.contains("replace this with a lookup table")),
        "{messages:#?}"
    );
    config.rules[0].include = vec!["missing.json".to_string()];
    assert!(super::check_with_files(&root, &config, &files)
        .unwrap()
        .is_empty());
}

#[test]
fn jsonc_catalog_comments_support_standard_suppression_directives() {
    for fixture in ["line-disable", "next-line-disable", "file-disable"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-finite-text-columns/fixture")
            .join(fixture);
        let file = root.join("schema.json");
        let mut config = NoMistakesConfig::default();
        config.rules.push(RuleDef {
            rule: super::RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("schemaCatalogPath: schema.json").unwrap(),
            ..RuleDef::default()
        });
        let mut findings =
            super::check_with_files(&root, &config, std::slice::from_ref(&file)).unwrap();
        if fixture == "file-disable" {
            assert_eq!(findings.len(), 1, "fixture {fixture}");
        } else {
            assert_eq!(findings.len(), 2, "fixture {fixture}");
            assert!(findings.iter().all(|finding| finding.line > 1));
        }
        let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
        super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
        if fixture == "file-disable" {
            assert!(findings.is_empty(), "fixture {fixture}: {findings:#?}");
        } else {
            assert_eq!(findings.len(), 1, "fixture {fixture}: {findings:#?}");
            assert_eq!(findings[0].target.as_deref(), Some("column:invoices.state"));
        }
    }
}
