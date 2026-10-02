use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

#[test]
fn invalid_fact_demands_fail_before_collection() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/identifier-length/standalone-prepared");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &[]);
    for (rule, options, expected) in [
        ("finite-set-consistency", "sets: false", "sets"),
        (
            "postgres-column-requires-trigger",
            "schemaCatalogPath: [schema.json]",
            "schemaCatalogPath",
        ),
        (
            "postgres-identifier-length",
            "sqlInclude: false",
            "sqlInclude",
        ),
        (
            "postgres-identifier-length",
            "sqlInclude: ['[']",
            "invalid sqlInclude",
        ),
        (
            "postgres-lock-ordering",
            "executorNames: write",
            "executorNames",
        ),
    ] {
        let mut config = NoMistakesConfig::default();
        config.rules.push(RuleDef {
            rule: rule.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..RuleDef::default()
        });
        let sources = snapshot.source_store_for(&root);
        let error = prepare_call_site_facts(&root, &config, &sources)
            .err()
            .expect("invalid demand must not produce partial facts");
        assert!(error.to_string().contains(expected), "{rule}: {error}");
        assert_eq!(sources.physical_read_count(), 0);
    }
}

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

#[test]
fn standalone_schema_and_catalog_rules_share_complete_fact_demand() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/identifier-length/standalone-prepared");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("schema.sql"), root.join("schema.json")];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let facts = prepare_call_site_facts(&root, &config, &sources)
        .unwrap()
        .unwrap();
    assert!(facts.postgres_schema_catalog("schema.json").is_ok());
    assert!(facts.postgres_schema_file(&files[0]).is_ok());
    assert_eq!(sources.physical_read_count(), 2);
    let findings =
        crate::codebase::rules::run_filesystem_rules_with_config(&root, &config, &files).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == "postgres-identifier-length"
                && finding.target.as_deref() == Some("very_long_column_name")),
        "{findings:#?}"
    );
}
