use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/status-lifecycle")
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

// Output cases moved to cli_postgres_snapshot_status_lifecycle.

#[test]
fn option_errors_name_the_field() {
    let root = fixture();
    let err = |options: &str| {
        check_with_files(&root, &config(options), &[root.join("schema.json")])
            .unwrap_err()
            .to_string()
    };
    assert!(err("lifecycleVerbs: [sent]\n").contains("option schemaCatalogPath: required"));
    assert!(err("schemaCatalogPath: \" \"\n").contains("option schemaCatalogPath: required"));
    assert!(
        err("schemaCatalogPath: schema.json\n").contains("option statusColumns: must not be empty")
    );
    assert!(err("schemaCatalogPath: schema.json\nstatusColumns: []\n")
        .contains("option statusColumns: must not be empty"));
    assert!(err("schemaCatalogPath: schema.json\nstatusColumns: ['']\n")
        .contains("option statusColumns: empty string"));
    assert!(
        err("schemaCatalogPath: schema.json\nstatusColumns: [status, status]\n")
            .contains("option statusColumns: duplicate entry status")
    );
    assert!(
        err("schemaCatalogPath: schema.json\nstatusColumns: [status]\nlifecycleVerbs: ['']\n")
            .contains("option lifecycleVerbs: empty string")
    );
    assert!(err(
        "schemaCatalogPath: schema.json\nstatusColumns: [status]\nlifecycleVerbs: [sent, sent]\n"
    )
    .contains("option lifecycleVerbs: duplicate entry sent"));
    assert!(
        err("schemaCatalogPath: schema.json\nminLifecycleColumns: 0\n")
            .contains("option minLifecycleColumns: must be at least 1")
    );
    assert!(
        err("schemaCatalogPath: schema.json\nminLifecycleColumns: -1\n")
            .contains("option minLifecycleColumns: must be at least 1")
    );
    assert!(err(
        "schemaCatalogPath: schema.json\nstatusColumns: [status]\nallow:\n  - object: table:orders\n    reason: \" \"\n"
    )
    .contains("option allow: entry table:orders needs a reason"));
    assert!(
        err("schemaCatalogPath: schema.json\nstatusColumns: [status]\nallow:\n  - object: nope\n    reason: why\n")
            .contains("option allow: invalid object ref nope")
    );
    assert!(err(
        "schemaCatalogPath: schema.json\nstatusColumns: [status]\nallow:\n  - object: table:orders\n    reason: one\n  - object: table:orders\n    reason: two\n"
    )
    .contains("option allow: duplicate entry table:orders"));
}
