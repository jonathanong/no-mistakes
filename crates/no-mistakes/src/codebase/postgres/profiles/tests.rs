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
    assert_eq!(profiles[0].import_specifier, "");
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
            "postgres-bounded-statements",
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

#[test]
fn bounded_rule_alone_requests_bounds_and_standalone_preparation_keeps_them() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture/prepared-demand");
    let file = root.join("queries.sql");
    let files = vec![file.clone()];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    for (rule, expected) in [
        ("postgres-required-predicates", false),
        ("postgres-sql-shape-policy", false),
        ("postgres-bounded-statements", true),
    ] {
        let config = rule_config(rule);
        let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
        configure_prepared_postgres_plan(&config, &mut plan).unwrap();
        assert!(plan.postgres_dml);
        assert_eq!(plan.postgres_bounds, expected, "{rule}");
        let prepared = prepare_rule_sql_facts(
            &root,
            &files,
            std::sync::Arc::clone(&sources),
            &config,
            &[rule],
        )
        .unwrap();
        let statements = prepared.postgres.statements(&file, None).unwrap();
        assert_eq!(
            statements[0].bounds.len(),
            if expected { 3 } else { 0 },
            "{rule}"
        );
    }
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn every_embedded_rule_requires_explicit_executor_selection() {
    use crate::codebase::postgres::extract_embedded_sql_from_source;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/string-literal.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut ids = PREPARED_EMBEDDED_SQL_RULE_IDS.to_vec();
    ids.extend([
        "postgres-idempotent-insert",
        "postgres-require-query-annotation",
    ]);
    for rule_id in ids {
        for (yaml, expected_calls) in [
            ("importSpecifier: '@example/db'", 1),
            ("executorNames: [query]", 1),
            // Explicit empty names opt out: SQL files and native SQL only.
            ("executorNames: []", 0),
            ("importSpecifier: '@other/db'", 0),
        ] {
            let mut config = NoMistakesConfig::default();
            config.rules.push(RuleDef {
                rule: rule_id.to_string(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str(yaml).unwrap(),
                ..RuleDef::default()
            });
            let profiles = configured_embedded_sql_options(&config, &[rule_id]).unwrap();
            let facts = extract_embedded_sql_from_source(&path, &source, &profiles[0]);
            assert_eq!(facts.calls.len(), expected_calls, "{rule_id}: {yaml}");
        }
    }
}

#[test]
fn every_embedded_rule_rejects_an_absent_executor_selection() {
    let mut ids = PREPARED_EMBEDDED_SQL_RULE_IDS.to_vec();
    ids.extend([
        "postgres-idempotent-insert",
        "postgres-require-query-annotation",
    ]);
    for rule_id in ids {
        // A blank module is absent too, and so is an options mapping that
        // only carries unrelated keys.
        for yaml in ["{}", "importSpecifier: ''", "sqlInclude: ['**/*.sql']"] {
            let mut config = NoMistakesConfig::default();
            config.rules.push(RuleDef {
                rule: rule_id.to_string(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str(yaml).unwrap(),
                ..RuleDef::default()
            });
            let error = configured_embedded_sql_options(&config, &[rule_id])
                .expect_err("an absent executor selection must be a configuration error");
            let message = error.to_string();
            assert!(
                message.starts_with(&format!("{rule_id} option importSpecifier: set ")),
                "{rule_id}: {yaml}: {message}"
            );
            assert!(message.contains("executorNames: []"), "{message}");
            assert!(
                message.contains("docs/migrations/explicit-postgres-executors.md"),
                "{message}"
            );
        }
    }
}
