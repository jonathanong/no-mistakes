use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

#[test]
fn schema_catalog_rules_prepare_a_catalog_without_call_sites() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/column-requires-trigger");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-column-requires-trigger".to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("schemaCatalogPath: schema.json\n").unwrap(),
        ..RuleDef::default()
    });
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
        &root,
        &[root.join("schema.json")],
    );
    let sources = snapshot.source_store_for(&root);
    let facts = prepare_call_site_facts(&root, &config, &sources)
        .unwrap()
        .expect("catalog demand");
    assert!(facts.postgres_schema_catalog("schema.json").is_ok());
}
