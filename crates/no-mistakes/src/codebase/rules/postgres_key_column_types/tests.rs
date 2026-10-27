use super::RULE_ID;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

mod behavior;
mod config;
mod entrypoint;
mod support;

#[test]
fn catalog_scan_errors_propagate_through_the_rule_entrypoint() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
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
