use super::super::{check_with_files_and_sources, check_with_files_sources_and_facts, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/key-column-types/catalogs")
}

fn config(options: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn prepared_error(config: &NoMistakesConfig) -> String {
    let files = Vec::new();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    check_with_files_sources_and_facts(
        &root(),
        config,
        &files,
        sources.as_ref(),
        &CheckFactMap::default(),
    )
    .unwrap_err()
    .to_string()
}

#[test]
fn propagates_catalog_path_option_conversion_errors() {
    let config = config("schemaCatalogPath: 42\nallowedTypes: [uuid]\n");
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let error = check_with_files_and_sources(&root(), &config, &[], &sources).unwrap_err();
    assert!(error.to_string().contains("schemaCatalogPath"), "{error:#}");
}

#[test]
fn propagates_rule_option_compile_filter_and_catalog_errors() {
    let error = prepared_error(&config(
        "schemaCatalogPath: mixed.json\nallowedTypes: uuid\n",
    ));
    assert!(error.contains("allowedTypes"), "{error}");

    let error = prepared_error(&config("schemaCatalogPath: mixed.json\nallowedTypes: []\n"));
    assert!(error.contains("required and nonempty"), "{error}");

    let mut invalid_filter_config = config("schemaCatalogPath: mixed.json\nallowedTypes: [uuid]\n");
    invalid_filter_config.rules[0].include = vec!["[".to_string()];
    let error = prepared_error(&invalid_filter_config);
    assert!(error.contains("include contains invalid glob"), "{error}");

    let error = prepared_error(&config(
        "schemaCatalogPath: missing.json\nallowedTypes: [uuid]\n",
    ));
    assert!(error.contains("missing.json"), "{error}");
}
