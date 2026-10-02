use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

#[test]
fn profiles_apply_defaults_and_deduplicate_executor_order() {
    let mut config = NoMistakesConfig::default();
    for options in [
        "executorNames: [write, read]",
        "executorNames: [read, write, read]",
    ] {
        config.rules.push(RuleDef {
            rule: "postgres-lock-ordering".to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..RuleDef::default()
        });
    }
    let profiles = configured_embedded_sql_options(
        &config,
        &["postgres-lock-ordering", "postgres-conflict-ordering"],
    )
    .unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].import_specifier, "@data-stores/psql");
    assert_eq!(profiles[0].executor_names, ["read", "write"]);
}

#[test]
fn profiles_reject_invalid_shared_option_shapes() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-lock-ordering".to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("executorNames: write").unwrap(),
        ..RuleDef::default()
    });
    let error = configured_embedded_sql_options(&config, &["postgres-lock-ordering"])
        .expect_err("a scalar executorNames value must be rejected");
    assert!(error.to_string().contains("executorNames"));
}

#[test]
fn catalog_paths_are_normalized_and_deduplicated() {
    let mut config = NoMistakesConfig::default();
    for path in ["schema.json", "./schema.json"] {
        config.rules.push(RuleDef {
            rule: "postgres-lock-ordering".to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!("schemaCatalogPath: {path}")).unwrap(),
            ..RuleDef::default()
        });
    }
    assert_eq!(
        configured_schema_catalog_paths(&config, &["postgres-lock-ordering"]).unwrap(),
        ["schema.json"]
    );
}

#[test]
fn schema_catalog_paths_follow_the_supplied_rule_id() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "schema-catalog-test-rule".to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("schemaCatalogPath: db/schema.json").unwrap(),
        ..RuleDef::default()
    });
    assert_eq!(
        configured_schema_catalog_paths(&config, &["schema-catalog-test-rule"]).unwrap(),
        ["db/schema.json"]
    );
    assert_eq!(
        PREPARED_EMBEDDED_SQL_RULE_IDS,
        [
            "postgres-conflict-ordering",
            "postgres-lock-ordering",
            "postgres-required-predicates",
            "postgres-generated-column-predicates",
            "postgres-explicit-columns",
            "postgres-no-offset",
            "postgres-sql-shape-policy",
            "postgres-no-generated-column-writes",
        ]
    );
    assert_eq!(
        SCHEMA_CATALOG_RULE_IDS,
        [
            "postgres-column-requires-trigger",
            "postgres-conflict-ordering",
            "postgres-lock-ordering",
            "postgres-required-comments",
            "postgres-duplicate-function-body",
            "postgres-table-shape",
            "postgres-status-with-lifecycle-timestamps",
            "postgres-object-naming",
            "postgres-column-naming",
            "postgres-finite-text-columns",
            "postgres-array-columns",
            "postgres-required-predicates",
            "postgres-explicit-columns",
        ]
    );
}

#[test]
fn standalone_schema_preparation_rejects_invalid_options_before_reads() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/identifier-length/standalone-prepared");
    let files = vec![root.join("schema.sql")];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    for (options, expected) in [
        ("sqlInclude: ['[']", "invalid sqlInclude"),
        ("sqlInclude: false", "sqlInclude"),
        ("schemaCatalogPath: [schema.json]", "schemaCatalogPath"),
    ] {
        let sources = snapshot.source_store_for(&root);
        let mut config = NoMistakesConfig::default();
        config.rules.push(RuleDef {
            rule: "postgres-identifier-length".to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..RuleDef::default()
        });
        let error = prepare_rule_sql_facts(
            &root,
            &files,
            std::sync::Arc::clone(&sources),
            &config,
            &["postgres-identifier-length"],
        )
        .err()
        .expect("invalid options must fail during preparation");
        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(sources.physical_read_count(), 0);
    }
}

#[test]
fn invalid_schema_and_statement_options_stop_request_planning() {
    for rule in ["postgres-identifier-length", "postgres-sql-shape-policy"] {
        let config = NoMistakesConfig {
            rules: vec![RuleDef {
                rule: rule.into(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str("sqlInclude: false").unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            configure_prepared_postgres_plan(&config, &mut Default::default())
                .unwrap_err()
                .to_string()
                .contains("sqlInclude")
        );
    }
}
