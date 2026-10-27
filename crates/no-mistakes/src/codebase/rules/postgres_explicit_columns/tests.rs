use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-explicit-columns/fixture")
            .join(name),
    )
}

pub(super) fn config_yaml(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn prepared_catalog_errors_when_the_fact_entry_is_broken_or_missing() {
    let root = fixture("edges");
    let sql = root.join("sql/qualified.sql");
    let sources = super::super::source_store_for_files(&[root.join("schema.json"), sql.clone()]);
    let mut broken = CheckFactMap::default();
    broken.postgres_schema_catalogs.insert(
        "missing/schema.json".to_string(),
        Err(Arc::<str>::from("catalog broke")),
    );
    let yaml = "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: missing/schema.json\n";
    let error = check_with_files_sources_and_facts(
        &root,
        &config_yaml(yaml),
        std::slice::from_ref(&sql),
        &sources,
        &broken,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("catalog broke"), "{error}");
    let missing = check_with_files_sources_and_facts(
        &root,
        &config_yaml(
            "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\nrelations: [accounts]\n",
        ),
        std::slice::from_ref(&sql),
        &sources,
        &CheckFactMap::default(),
    )
    .unwrap_err();
    assert!(missing
        .to_string()
        .contains("prepared schema catalog is missing"));
}

#[test]
fn invalid_include_glob_is_a_config_error() {
    let root = fixture("edges");
    let error = check_with_files(
        &root,
        &config_yaml("schemaCatalogPath: schema.json\ninclude: ['[']\n"),
        &[],
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("invalid glob"), "{error}");
}
