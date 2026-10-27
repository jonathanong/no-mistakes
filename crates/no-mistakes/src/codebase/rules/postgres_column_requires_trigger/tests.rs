use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/column-requires-trigger")
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

#[test]
fn option_errors_name_the_field() {
    let cases = [
        (
            "requirements:\n  - column: updated_at\n    function: fn_touch_updated_at\n",
            "option schemaCatalogPath: required",
        ),
        (
            "schemaCatalogPath: schema.json\nrequirements:\n  - column: ' '\n    function: fn\n",
            "option column: required",
        ),
        (
            "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: ' '\n",
            "option function: required",
        ),
        (
            "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn\n    timing: whenever\n",
            "option timing: unknown value whenever",
        ),
        (
            "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn\n    events: []\n",
            "option events: must not be empty",
        ),
        (
            "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn\n    events: [merge]\n",
            "option events: unknown event merge",
        ),
        (
            "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn\n    events: [truncate]\n",
            "option forEachRow: truncate triggers are FOR EACH STATEMENT",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: table:orders\n    reason: ' '\n",
            "option allow: entry table:orders needs a reason",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: nope\n    reason: because\n",
            "option allow: invalid object ref nope",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: table:orders\n    reason: one\n  - object: table:orders\n    reason: two\n",
            "option allow: duplicate entry table:orders",
        ),
    ];
    for (options, expected) in cases {
        let error = check_with_files(&fixture(), &config(options), &[]).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "{expected} not in {error}"
        );
    }
}
