#[test]
fn catalog_paths_are_normalized_and_deduplicated() {
    let mut config = NoMistakesConfig::default();
    for path in ["schema.json", "./schema.json"] {
        config.rules.push(RuleDef {
            rule: "postgres-lock-ordering".to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!("schemaCatalogPath: {path}\nexecutorNames: []"))
                .unwrap(),
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
            "postgres-idempotent-insert",
            "postgres-conflict-ordering",
            "postgres-lock-ordering",
            "postgres-required-predicates",
            "postgres-generated-column-predicates",
            "postgres-explicit-columns",
            "postgres-no-test-state-mutation",
            "postgres-bounded-statements",
            "postgres-no-offset",
        "postgres-no-transformed-uuid-predicates",
            "postgres-no-mutable-sql-registry",
            "postgres-no-unbounded-sql-identifier",
            "postgres-sql-shape-policy",
            "postgres-sql-statement-policy",
            "postgres-no-generated-column-writes",
            "postgres-require-query-annotation",
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
            "postgres-key-column-types",
            "postgres-no-transformed-uuid-predicates",
            "postgres-required-predicates",
            "postgres-explicit-columns",
            "postgres-bounded-statements",
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
fn shape_only_preparation_does_not_extract_schema_facts() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/shape-only-schema");
    let file = root.join("schema.sql");
    let files = vec![file.clone()];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let shape = prepare_rule_sql_facts(
        &root,
        &files,
        std::sync::Arc::clone(&sources),
        &rule_config("postgres-sql-shape-policy"),
        &["postgres-sql-shape-policy"],
    )
    .unwrap();
    assert!(shape.postgres.schema.is_empty());
    let statements = shape.postgres.statements(&file, None).unwrap();
    assert!(statements.iter().any(|facts| {
        facts
            .selects
            .iter()
            .any(|select| !select.not_in_subqueries.is_empty())
    }));
    let identified = prepare_rule_sql_facts(
        &root,
        &files,
        sources,
        &rule_config("postgres-identifier-length"),
        &["postgres-identifier-length"],
    )
    .unwrap();
    assert!(!identified.postgres.schema(&file).unwrap().tables.is_empty());
}

fn rule_config(rule: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: rule.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("sqlInclude: ['**/*.sql']\nexecutorNames: []").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}
