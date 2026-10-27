use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/table-shape")
}

// Output cases moved to cli_postgres_snapshot_table_shape.

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
        ("shapes: []\n", "option schemaCatalogPath: required"),
        ("schemaCatalogPath: ' '\n", "option schemaCatalogPath: required"),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: ' '\n    tablePattern: x\n",
            "option name: required",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: same\n    tablePattern: a\n  - name: same\n    tablePattern: b\n",
            "option name: duplicate entry same",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: ''\n",
            "option tablePattern: required",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: '('\n",
            "option tablePattern: invalid regex (:",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - {}\n",
            "option requiredColumns: exactly one of name or namePattern",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - name: id\n        namePattern: x\n",
            "option requiredColumns: exactly one of name or namePattern",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - name: id\n        references: []\n",
            "option references: must not be empty",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - name: id\n        references: [' ']\n",
            "option references: blank table",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - name: ' '\n",
            "option requiredColumns: exactly one of name or namePattern",
        ),
        (
            "schemaCatalogPath: schema.json\nbannedTablePatterns:\n  - pattern: ' '\n    message: because\n",
            "option pattern: required",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - name: id\n        onDelete: whenever\n",
            "option onDelete: unknown value whenever",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - name: id\n        foreignKey: false\n        onDelete: cascade\n",
            "option foreignKey: false conflicts with references or onDelete",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    primaryKeyTypes: ['']\n",
            "option primaryKeyTypes: empty string",
        ),
        (
            "schemaCatalogPath: schema.json\nbannedTablePatterns:\n  - pattern: '('\n    message: because\n",
            "option pattern: invalid regex (:",
        ),
        (
            "schemaCatalogPath: schema.json\nbannedTablePatterns:\n  - pattern: x\n    message: ' '\n",
            "option message: required",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredTriggers:\n      - function: ' '\n",
            "option function: required",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredTriggers:\n      - function: fn\n        timing: whenever\n",
            "option timing: unknown value whenever",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredTriggers:\n      - function: fn\n        events: []\n",
            "option events: must not be empty",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredTriggers:\n      - function: fn\n        events: [merge]\n",
            "option events: unknown event merge",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredColumns:\n      - name: id\n        type: ' '\n",
            "option type: required",
        ),
        (
            "schemaCatalogPath: schema.json\nshapes:\n  - name: s\n    tablePattern: t\n    requiredTriggers:\n      - function: fn\n        events: [truncate]\n",
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
