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

const TOUCH: &str = "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn_touch_updated_at\n";

#[test]
fn reports_every_specified_finding_text() {
    let body = texts(TOUCH);
    assert!(body.contains(&"table has column updated_at but no BEFORE UPDATE FOR EACH ROW trigger executing fn_touch_updated_at()".to_string()));
    assert!(body.contains(&"trigger trigger_orders_touch executes fn_touch_updated_at() only for UPDATE OF status; it must fire on every UPDATE".to_string()));
    assert!(body.contains(&"trigger trigger_orders_touch executes fn_touch_updated_at() but the table has no updated_at column".to_string()));
    assert!(!body
        .iter()
        .any(|text| text.contains("table:orders") || text.contains("orders:")));
    assert!(!body.iter().any(|text| text.contains("order_events")
        || text.contains("invoices")
        || text.contains("events")));
}

#[test]
fn schema_qualified_insert_or_update_and_partitioned_tables_pass() {
    let body = messages(TOUCH).join("\n");
    assert!(!body.contains("table:orders:"));
    assert!(!body.contains("table:invoices:"));
    assert!(!body.contains("table:events:"));
    assert!(!body.contains("table:order_events:"));
}

#[test]
fn wrong_timing_and_statement_triggers_are_missing() {
    let body = texts(TOUCH);
    let missing = body
        .iter()
        .filter(|text| text.starts_with("table has column updated_at"))
        .count();
    assert!(missing >= 2, "{body:?}");
    let joined = messages(TOUCH).join("\n");
    assert!(joined.contains("table:conditional_orders:"));
    assert!(joined.contains("table:late_orders:"));
    assert!(joined.contains("table:statement_orders:"));
    assert!(joined.contains("table:import_rows:"));
}

#[test]
fn two_requirements_are_checked_independently() {
    let options = "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn_touch_updated_at\n  - column: deleted_at\n    function: fn_touch_deleted_at\n";
    let joined = messages(options).join("\n");
    assert!(joined.contains("table has column deleted_at but no BEFORE UPDATE FOR EACH ROW trigger executing fn_touch_deleted_at()"));
    assert!(joined.contains("table:both_cols:"));
}

#[test]
fn column_list_can_be_allowed_and_multi_event_timing_is_rendered() {
    let allowed = "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn_touch_updated_at\n    allowColumnList: true\n";
    assert!(!messages(allowed).join("\n").contains("partial_orders"));
    let events = "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn_touch_updated_at\n    timing: instead-of\n    events: [update, delete]\n    forEachRow: false\n";
    let joined = messages(events).join("\n");
    assert!(joined.contains(
        "no INSTEAD OF UPDATE OR DELETE FOR EACH STATEMENT trigger executing fn_touch_updated_at()"
    ));
}

#[test]
fn repeated_runs_are_identical() {
    assert_eq!(messages(TOUCH), messages(TOUCH));
}

#[test]
fn empty_requirements_report_nothing() {
    assert!(messages("schemaCatalogPath: schema.json\n").is_empty());
}

#[test]
fn allow_suppresses_a_table_and_reports_stale_entries() {
    let options = "schemaCatalogPath: schema.json\nrequirements:\n  - column: updated_at\n    function: fn_touch_updated_at\nallow:\n  - object: table:import_rows\n    reason: updated_at is written explicitly by the importer\n  - object: table:order_events\n    reason: not a finding\n";
    let joined = messages(options).join("\n");
    assert!(!joined.contains("table:import_rows:"));
    assert!(
        joined.contains("stale postgres-column-requires-trigger allow entry: table:order_events")
    );
}

#[test]
fn function_names_fold_unquoted_identifiers_and_keep_quoted_case() {
    assert_eq!(normalize_function_name("public.Fn_Touch"), "fn_touch");
    assert_eq!(normalize_function_name("public.\"Touch\""), "Touch");
    assert_eq!(
        normalize_function_name("public.\"touch.updated\""),
        "touch.updated"
    );
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
