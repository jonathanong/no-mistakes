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

fn messages(options: &str) -> Vec<String> {
    let root = fixture();
    check_with_files(&root, &config(options), &[root.join("schema.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn texts(options: &str) -> Vec<String> {
    messages(options)
        .into_iter()
        .map(|message| {
            message
                .split(": ")
                .nth(2)
                .unwrap_or(message.as_str())
                .to_string()
        })
        .collect()
}

const CATALOG: &str = "schemaCatalogPath: schema.json\n";

const ALL: &str = "schemaCatalogPath: schema.json\nminLength: 10\nobjects: [table, view, materialized-view, column]\nexemptColumnNamePatterns: ['^(id|created_at|updated_at)$', '^vendor_']\n";

#[test]
fn default_objects_report_missing_and_whitespace_table_comments() {
    let joined = messages(CATALOG).join("\n");
    assert!(joined.contains("table:events: table has no COMMENT ON TABLE"));
    assert!(joined.contains("table:blank_rows: table has no COMMENT ON TABLE"));
    assert!(joined.contains("table:schema_migrations: table has no COMMENT ON TABLE"));
    assert!(!joined.contains("table:orders:"));
    assert!(!joined.contains("table:short_rows:"));
    assert!(!joined.contains("column:"));
    assert!(!joined.contains("view:"));
    assert!(!joined.contains("materialized-view:"));
}

#[test]
fn short_table_comment_uses_configured_min_length() {
    let options = "schemaCatalogPath: schema.json\nminLength: 10\n";
    let joined = messages(options).join("\n");
    assert!(joined.contains("table:short_rows: table comment is shorter than 10 characters"));
    assert!(joined.contains("table:blank_rows: table has no COMMENT ON TABLE"));
    assert!(!joined.contains("table:orders:"));
}

#[test]
fn reports_every_object_kind_and_finding_text() {
    let joined = messages(ALL).join("\n");
    assert!(joined.contains("table:events: table has no COMMENT ON TABLE"));
    assert!(joined.contains("table:short_rows: table comment is shorter than 10 characters"));
    assert!(joined.contains("column:orders.secret: column has no COMMENT ON COLUMN"));
    assert!(joined.contains("column:orders.code: column comment is shorter than 10 characters"));
    assert!(joined.contains("column:orders.title: column has no COMMENT ON COLUMN"));
    assert!(joined.contains("view:open_orders: view has no COMMENT ON VIEW"));
    assert!(joined.contains("view:blank_view: view has no COMMENT ON VIEW"));
    assert!(joined.contains("view:tiny_view: view comment is shorter than 10 characters"));
    assert!(joined.contains(
        "materialized-view:paid_orders: materialized view has no COMMENT ON MATERIALIZED VIEW"
    ));
    assert!(!joined.contains("table:orders:"));
    assert!(!joined.contains("column:orders.note:"));
    assert!(!joined.contains("column:orders.id:"));
    assert!(!joined.contains("column:orders.vendor_id:"));
    assert!(!joined.contains("column:events.payload:"));
    assert!(!joined.contains("view:titled_orders:"));
}

#[test]
fn column_name_patterns_select_and_exempt_still_wins() {
    let options = "schemaCatalogPath: schema.json\nminLength: 10\nobjects: [column]\ncolumnNamePatterns: ['^note$', '^secret$', '^vendor_', '^code$', '^title$']\nexemptColumnNamePatterns: ['^vendor_']\n";
    let joined = messages(options).join("\n");
    assert!(joined.contains("column:orders.secret:"));
    assert!(joined.contains("column:orders.code:"));
    assert!(joined.contains("column:orders.title:"));
    assert!(!joined.contains("column:orders.note:"));
    assert!(!joined.contains("column:orders.vendor_id:"));
    assert!(!joined.contains("column:orders.id:"));
    assert!(!joined.contains("column:events.payload:"));
    assert!(!joined.contains("column:schema_migrations.version:"));
}

#[test]
fn empty_column_patterns_check_every_non_exempt_column() {
    let options = "schemaCatalogPath: schema.json\nobjects: [column]\nexemptColumnNamePatterns: ['^(id|created_at|updated_at)$', '^vendor_']\n";
    let joined = messages(options).join("\n");
    assert!(joined.contains("column:orders.secret:"));
    assert!(joined.contains("column:orders.title:"));
    assert!(joined.contains("column:schema_migrations.version:"));
    assert!(!joined.contains("column:orders.note:"));
    assert!(!joined.contains("column:orders.code:"));
    assert!(!joined.contains("column:orders.id:"));
    assert!(!joined.contains("column:orders.vendor_id:"));
    assert!(!joined.contains("column:events.payload:"));
    assert!(!joined.contains("column:blank_rows.id:"));
}

#[test]
fn partitioned_parent_column_comment_counts() {
    let options =
        "schemaCatalogPath: schema.json\nobjects: [column]\ncolumnNamePatterns: ['^payload$']\n";
    assert!(messages(options).is_empty());
    let tables = messages("schemaCatalogPath: schema.json\n").join("\n");
    assert!(tables.contains("table:events:"));
}

#[test]
fn views_are_separate_from_materialized_views() {
    let views = messages("schemaCatalogPath: schema.json\nobjects: [view]\n").join("\n");
    assert!(views.contains("view:open_orders:"));
    assert!(views.contains("view:blank_view:"));
    assert!(!views.contains("view:tiny_view:"));
    assert!(!views.contains("materialized-view:"));
    let materialized =
        messages("schemaCatalogPath: schema.json\nobjects: [materialized-view]\n").join("\n");
    assert!(materialized.contains("materialized-view:paid_orders:"));
    assert!(!materialized.contains("view:open_orders:"));
    assert!(!materialized.contains("view:blank_view:"));
    assert!(!materialized.contains("view:tiny_view:"));
}

#[test]
fn repeated_runs_are_identical() {
    assert_eq!(messages(ALL), messages(ALL));
}

#[test]
fn allow_suppresses_findings_and_reports_stale_entries() {
    let options = "schemaCatalogPath: schema.json\nminLength: 10\nobjects: [table, view, materialized-view, column]\nexemptColumnNamePatterns: ['^(id|created_at|updated_at)$', '^vendor_']\nallow:\n  - object: table:schema_migrations\n    reason: Owned by the migration runner\n  - object: column:orders.secret\n    reason: internal\n  - object: view:open_orders\n    reason: documented in the app\n  - object: materialized-view:paid_orders\n    reason: documented in the app\n  - object: table:not_real\n    reason: not a finding\n";
    let joined = messages(options).join("\n");
    assert!(!joined.contains("table:schema_migrations:"));
    assert!(!joined.contains("column:orders.secret:"));
    assert!(!joined.contains("view:open_orders:"));
    assert!(!joined.contains("materialized-view:paid_orders:"));
    assert!(joined.contains("stale postgres-required-comments allow entry: table:not_real"));
    assert!(joined.contains("table:events:"));
}

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

#[test]
fn finding_text_helpers_cover_each_listed_sentence() {
    let body = texts(ALL);
    for text in [
        "table has no COMMENT ON TABLE",
        "table comment is shorter than 10 characters",
        "column has no COMMENT ON COLUMN",
        "view has no COMMENT ON VIEW",
        "materialized view has no COMMENT ON MATERIALIZED VIEW",
    ] {
        assert!(body.iter().any(|line| line == text), "missing {text}");
    }
}
