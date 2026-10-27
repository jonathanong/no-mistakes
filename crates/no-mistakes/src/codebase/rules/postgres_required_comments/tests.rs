use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/required-comments")
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

// Output cases moved to cli_postgres_snapshot_required_comments.

#[test]
fn option_errors_name_the_field() {
    let cases = [
        ("objects: [table]\n", "option schemaCatalogPath: required"),
        (
            "schemaCatalogPath: schema.json\nobjects: []\n",
            "option objects: must not be empty",
        ),
        (
            "schemaCatalogPath: schema.json\nobjects: [relation]\n",
            "option objects: unknown value relation",
        ),
        (
            "schemaCatalogPath: schema.json\nobjects: [table, table]\n",
            "option objects: duplicate entry table",
        ),
        (
            "schemaCatalogPath: schema.json\ncolumnNamePatterns: ['(']\n",
            "option columnNamePatterns: invalid regex (:",
        ),
        (
            "schemaCatalogPath: schema.json\nexemptColumnNamePatterns: ['(']\n",
            "option exemptColumnNamePatterns: invalid regex (:",
        ),
        (
            "schemaCatalogPath: schema.json\nminLength: 0\n",
            "option minLength: must be at least 1",
        ),
        (
            "schemaCatalogPath: schema.json\nminLength: -3\n",
            "option minLength: must be at least 1",
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
