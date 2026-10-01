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

fn messages(options: &str) -> Vec<String> {
    let root = fixture();
    check_with_files(&root, &config(options), &[root.join("schema.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn main_options() -> String {
    std::fs::read_to_string(fixture().join("options.yml")).unwrap()
}

fn joined(options: &str) -> String {
    messages(options).join("\n")
}

fn columns() -> &'static str {
    "statusColumns: [status, state]"
}

fn verbs() -> &'static str {
    "lifecycleVerbs: [sent, failed, paid, started, finished, bounced]"
}

const ADVICE: &str = "keep one source of truth: make status a GENERATED ALWAYS AS (...) STORED column computed from the timestamps, or, if rows return to earlier states, record each attempt or change in a history table";

#[test]
fn reports_stored_status_beside_lifecycle_timestamps() {
    let body = joined(&main_options());
    assert!(body.contains(&format!(
        "table:deliveries: table stores status column state next to lifecycle timestamps sent_at, failed_at; {ADVICE}"
    )));
    assert!(body.contains(&format!(
        "table:payments: table stores status column status next to lifecycle timestamps payment_failed_at, paid_at; {ADVICE}"
    )));
    assert!(body.contains(
        "table:precise_events: table stores status column status next to lifecycle timestamps sent_at, failed_at;"
    ));
    assert!(body.contains(&format!(
        "table:both_names: table stores status column state next to lifecycle timestamps started_at, finished_at; {ADVICE}"
    )));
    assert!(!body.contains("table:both_names: table stores status column status"));
    assert!(body.contains(&format!(
        "table:generated_state: table stores status column state next to lifecycle timestamps sent_at, bounced_at; {ADVICE}"
    )));
    assert!(!body.contains("table:generated_state: table stores status column status"));
}

#[test]
fn valid_shapes_and_non_lifecycle_columns_pass() {
    let body = joined(&main_options());
    for table in [
        "invoices",
        "invoice_drafts",
        "receipts",
        "orders",
        "dated_approvals",
        "preview_invoices",
        "resent_jobs",
        "payment_intents",
        "projected_events",
        "event_lists",
        "status_history",
        "named_types",
    ] {
        assert!(
            !body.contains(&format!("table:{table}:")),
            "{table} in {body}"
        );
    }
}

#[test]
fn defaults_skip_empty_verbs_and_apply_the_minimum() {
    assert!(
        messages("schemaCatalogPath: schema.json\nstatusColumns: [status, state]\n").is_empty()
    );
    assert!(messages(
        "schemaCatalogPath: schema.json\nstatusColumns: [status, state]\nlifecycleVerbs: []\n"
    )
    .is_empty());
    let defaults = joined(&format!(
        "schemaCatalogPath: schema.json\n{}\n{}\n",
        columns(),
        verbs()
    ));
    assert!(defaults.contains("table:deliveries:"));
    assert!(defaults.contains("table:payments:"));
    assert!(!defaults.contains("table:invoice_drafts:"));
    let status_only = joined(&format!(
        "schemaCatalogPath: schema.json\nstatusColumns: [status]\n{}\n",
        verbs()
    ));
    assert!(status_only.contains("table:payments:"));
    assert!(!status_only.contains("table:deliveries:"));
    let one = joined(&format!(
        "schemaCatalogPath: schema.json\n{}\nminLifecycleColumns: 1\n{}\n",
        columns(),
        verbs()
    ));
    assert!(one.contains(
        "table:invoice_drafts: table stores status column status next to lifecycle timestamps sent_at;"
    ));
    let three = joined(&format!(
        "schemaCatalogPath: schema.json\n{}\nminLifecycleColumns: 3\n{}\n",
        columns(),
        verbs()
    ));
    assert!(!three.contains("table:deliveries:"));
}

#[test]
fn allow_suppresses_a_table_and_runs_match() {
    let options = format!(
        "schemaCatalogPath: schema.json\n{}\n{}\nallow:\n  - object: table:deliveries\n    reason: provider mirror\n",
        columns(),
        verbs()
    );
    let body = joined(&options);
    assert!(!body.contains("table:deliveries: table stores"));
    assert!(body.contains("table:payments:"));
    let stale = joined(
        "schemaCatalogPath: schema.json\nstatusColumns: [status]\nlifecycleVerbs: [sent, failed]\nallow:\n  - object: table:missing\n    reason: gone\n",
    );
    assert!(stale
        .contains("stale postgres-status-with-lifecycle-timestamps allow entry: table:missing"));
    assert_eq!(messages(&main_options()), messages(&main_options()));
}

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

#[test]
fn custom_message_and_include_filter_the_catalog() {
    let mut configured = config(
        "schemaCatalogPath: schema.json\nstatusColumns: [state]\nlifecycleVerbs: [sent, failed]\n",
    );
    configured.rules[0].message = Some("use one source of truth".to_string());
    let root = fixture();
    let messages = check_with_files(&root, &configured, &[root.join("schema.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|message| { message.contains("table:deliveries: use one source of truth") }));
    configured.rules[0].include = vec!["missing.json".to_string()];
    assert!(
        check_with_files(&root, &configured, &[root.join("schema.json")])
            .unwrap()
            .is_empty()
    );
}
