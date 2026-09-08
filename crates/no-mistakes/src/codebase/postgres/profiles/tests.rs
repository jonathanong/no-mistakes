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
