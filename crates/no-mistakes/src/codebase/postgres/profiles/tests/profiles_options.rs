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
fn non_annotation_rules_do_not_request_annotation_projection() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-conflict-ordering".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("importSpecifier: '@example/db'\nexecutorNames: [write]")
            .unwrap(),
        ..RuleDef::default()
    });
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    configure_prepared_postgres_plan(&config, &mut plan).unwrap();
    assert!(!plan.query_annotation);
    assert!(plan.query_annotation_options.is_none());
}

#[test]
fn configured_annotation_demand_preserves_an_existing_all_profiles_request() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-require-query-annotation".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("importSpecifier: '@example/db'").unwrap(),
        ..RuleDef::default()
    });
    let mut plan = crate::codebase::check_facts::CheckFactPlan {
        query_annotation: true,
        ..Default::default()
    };
    configure_prepared_postgres_plan(&config, &mut plan).unwrap();
    assert!(plan.query_annotation_options.is_none());
}

#[test]
fn configured_annotation_demand_merges_an_existing_explicit_profile() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-require-query-annotation".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("importSpecifier: '@example/db'\nexecutorNames: [write]")
            .unwrap(),
        ..RuleDef::default()
    });
    let read = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    let write = EmbeddedSqlOptions::configured("@example/db", &["write".into()]);
    let mut plan = crate::codebase::check_facts::CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(vec![read.clone()]),
        ..Default::default()
    };
    configure_prepared_postgres_plan(&config, &mut plan).unwrap();
    assert!(plan.query_annotation);
    assert_eq!(plan.query_annotation_options, Some(vec![read, write]));
}

#[test]
fn standalone_annotation_rejects_malformed_executor_options() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-require-query-annotation".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("executorNames: false").unwrap(),
        ..RuleDef::default()
    });
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let files = Vec::new();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let error = prepare_rule_sql_facts(
        &root,
        &files,
        sources,
        &config,
        &["postgres-require-query-annotation"],
    )
    .err()
    .expect("malformed annotation options must be rejected");
    assert!(error.to_string().contains("executorNames"), "{error:#}");
}

#[test]
fn profiles_carry_scoped_executor_options_sorted_and_distinct() {
    let mut config = NoMistakesConfig::default();
    for options in [
        "importSpecifier: '@example/db'\nexecutorFactoryNames: [b, a]\nexecutorTypeNames: [T, T]",
        "importSpecifier: '@example/db'",
    ] {
        config.rules.push(RuleDef {
            rule: "postgres-lock-ordering".to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..RuleDef::default()
        });
    }
    let profiles = configured_embedded_sql_options(&config, &["postgres-lock-ordering"]).unwrap();
    // Scoped options make a distinct projection from the plain module profile.
    assert_eq!(profiles.len(), 2);
    let scoped = profiles
        .iter()
        .find(|profile| !profile.executor_factory_names.is_empty())
        .unwrap();
    assert_eq!(scoped.executor_factory_names, ["a", "b"]);
    assert_eq!(scoped.executor_type_names, ["T"]);
}

#[test]
fn profiles_carry_trusted_sql_tags_sorted_and_distinct() {
    let mut config = NoMistakesConfig::default();
    for options in [
        "importSpecifier: '@example/db'\ntrustedSqlTags:\n  - {module: '@example/db/sql', name: tag}\n  - {module: '@example/db', name: sql}\n  - {module: '@example/db', name: sql}",
        "importSpecifier: '@example/db'",
    ] {
        config.rules.push(RuleDef {
            rule: "postgres-lock-ordering".to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..RuleDef::default()
        });
    }
    let profiles = configured_embedded_sql_options(&config, &["postgres-lock-ordering"]).unwrap();
    assert_eq!(profiles.len(), 2);
    let tagged = profiles
        .iter()
        .find(|profile| !profile.trusted_sql_tags.is_empty())
        .unwrap();
    assert_eq!(
        tagged.trusted_sql_tags,
        vec![
            crate::codebase::postgres::TrustedSqlTag {
                module: "@example/db".into(),
                name: "sql".into(),
            },
            crate::codebase::postgres::TrustedSqlTag {
                module: "@example/db/sql".into(),
                name: "tag".into(),
            },
        ]
    );
    let mut scalar = NoMistakesConfig::default();
    scalar.rules.push(RuleDef {
        rule: "postgres-lock-ordering".to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("importSpecifier: '@example/db'\ntrustedSqlTags: sql")
            .unwrap(),
        ..RuleDef::default()
    });
    let error = configured_embedded_sql_options(&scalar, &["postgres-lock-ordering"])
        .expect_err("a scalar trustedSqlTags value must be rejected");
    assert!(error.to_string().contains("trustedSqlTags"), "{error}");
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

