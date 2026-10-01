use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/table-shape")
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

#[test]
fn valid_examples_and_edges_pass() {
    let body = joined(&main_options());
    for table in [
        "order_revisions",
        "orders",
        "article_votes",
        "thread_votes",
        "order_status_changes",
        "split_changes",
        "qualified_changes",
        "email_work_items",
        "domain_events",
        "report_export_cursors",
        "search_reindex_cursors",
        "billing_reconciliation_cursors",
        "only_revisions",
        "vendor_sync_history",
        "nullable_revisions",
        "ref_pattern",
    ] {
        assert!(
            !body.contains(&format!("table:{table}:")),
            "{table} in {body}"
        );
    }
    assert!(!body.contains("primary key column job"));
    assert!(!body.contains("table:draft_changes: foreign key on changed_by_id must reference"));
}

#[test]
fn reports_every_specified_finding_text() {
    let body = joined(&main_options());
    for text in [
        "table:missing_revisions: (shape revision-history) missing required column changes",
        "table:part_revisions: (shape revision-history) missing required column changes",
        "table:bad_revisions: (shape revision-history) column changes must be jsonb, found json",
        "table:bad_revisions: (shape revision-history) column changes must be NOT NULL",
        "table:bad_revisions: (shape revision-history) column revised_by_id must be a foreign key",
        "table:draft_changes: (shape change-ledger) column changed_by_id must be a foreign key",
        "table:cascade_revisions: (shape revision-history) foreign key on revised_by_id must be ON DELETE SET NULL, found CASCADE",
        "table:comment_votes: (shape per-target-votes) foreign key on user_id must be ON DELETE CASCADE, found SET NULL",
        "table:payment_status_changes: (shape change-ledger) foreign key on changed_by_id must be ON DELETE RESTRICT, found SET NULL",
        "table:invoice_status_changes: (shape change-ledger) foreign key on changed_by_id must reference one of deleted_user_identities, found accounts",
        "table:plain_revisions: (shape revision-history) no column matches _id$ with a foreign key",
        "table:bad_revisions: (shape revision-history) forbidden column updated_at",
        "table:content_votes: (shape per-target-votes) forbidden column target_table",
        "table:refund_status_changes: (shape change-ledger) forbidden column changed_at",
        "table:audit_events: (shape append-only-events) forbidden column updated_at",
        "table:flag_cursors: (shape job-cursor) forbidden column singleton",
        "table:bad_revisions: (shape revision-history) no BEFORE UPDATE OR DELETE FOR EACH ROW trigger executing fn_reject_mutation()",
        "table:order_history: table name matches banned pattern _(history|change_logs|audit_logs)$: use a *_revisions table",
        "table:sync_cursors: (shape job-cursor) primary key column cursor_name must be boolean or an enum, found text",
        "table:tenant_cursors: (shape job-cursor) primary key column tenant_id must be boolean or an enum, found uuid",
        "table:nopk_cursors: (shape job-cursor) table has no primary key",
        "table:bare_cursors: (shape job-cursor) no column matches ^cursor_|_through_at$",
    ] {
        assert!(body.contains(text), "missing {text}");
    }
}

#[test]
fn edge_cases_are_separate_from_the_main_contract() {
    let two = joined(
        "schemaCatalogPath: schema.json\nshapes:\n  - name: alpha\n    tablePattern: '^orders$'\n    requiredColumns:\n      - name: alpha\n  - name: beta\n    tablePattern: '^orders$'\n    requiredColumns:\n      - name: beta\nbannedTablePatterns:\n  - pattern: '^orders$'\n    message: not a domain table\n",
    );
    assert!(two.contains("(shape alpha) missing required column alpha"));
    assert!(two.contains("(shape beta) missing required column beta"));
    assert!(two.contains("banned pattern ^orders$: not a domain table"));
    let unchecked = joined(
        "schemaCatalogPath: schema.json\nshapes:\n  - name: job-cursor\n    tablePattern: '^sync_cursors$'\n    primaryKeyTypes: []\n    requiredColumns:\n      - name: updated_at\n",
    );
    assert!(unchecked.is_empty(), "{unchecked}");
    let pattern = joined(
        "schemaCatalogPath: schema.json\nshapes:\n  - name: ref-pattern\n    tablePattern: '^ref_pattern$'\n    requiredColumns:\n      - namePattern: '_id$'\n        references: [users, accounts]\n",
    );
    assert!(pattern.contains(
        "(shape ref-pattern) no column matches _id$ with a foreign key, references users, accounts"
    ));
    let statement = joined(
        "schemaCatalogPath: schema.json\nshapes:\n  - name: append-only-events\n    tablePattern: '^domain_events$'\n    requiredTriggers:\n      - function: fn_reject_mutation\n        events: [update, delete]\n        forEachRow: false\n",
    );
    assert!(statement.contains("FOR EACH STATEMENT trigger executing fn_reject_mutation()"));
    let nullable = joined(
        "schemaCatalogPath: schema.json\nshapes:\n  - name: revision-history\n    tablePattern: '^nullable_revisions$'\n    requiredColumns:\n      - name: revised_by_id\n        nullable: true\n",
    );
    assert!(nullable.contains("column revised_by_id must be nullable"));
}

#[test]
fn empty_shapes_report_nothing_and_runs_match() {
    assert!(messages("schemaCatalogPath: schema.json\n").is_empty());
    let options = main_options();
    assert_eq!(messages(&options), messages(&options));
}

#[test]
fn allow_suppresses_a_table_and_reports_stale_entries() {
    let body = joined(
        "schemaCatalogPath: schema.json\nbannedTablePatterns:\n  - pattern: '^order_history$'\n    message: banned\nallow:\n  - object: table:order_history\n    reason: kept\n  - object: table:missing\n    reason: not a finding\n",
    );
    assert!(!body.contains("table:order_history:"));
    assert!(body.contains("stale postgres-table-shape allow entry: table:missing"));
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

#[test]
fn column_lists_when_clauses_and_quoted_names_stay_distinct() {
    let options = "schemaCatalogPath: edges.json
shapes:
  - name: enum_key
    tablePattern: ^keyed$
    primaryKeyTypes: [enum]
  - name: empty
    tablePattern: ^empty_key$
    primaryKeyTypes: [uuid]
  - name: triggers
    tablePattern: ^partial$
    requiredTriggers:
      - function: fn_touch
        events: [update]
  - name: not_fk
    tablePattern: ^partial$
    requiredColumns:
      - name: owner_id
        foreignKey: false
  - name: quoted
    tablePattern: ^partial$
    requiredColumns:
      - name: owner_id
        references: ['\"audit.events\"']
  - name: other
    tablePattern: ^partial$
    requiredColumns:
      - name: owner_id
        references: ['\"other.events\"']
  - name: pattern
    tablePattern: ^partial$
    requiredColumns:
      - namePattern: ^absent$
        nullable: true
";
    let root = fixture();
    let body = check_with_files(&root, &config(options), &[root.join("edges.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!body.contains("keyed"), "{body}");
    assert!(
        body.contains("(shape empty) table has no primary key"),
        "{body}"
    );
    assert!(
        body.contains("(shape triggers) no BEFORE UPDATE FOR EACH ROW trigger"),
        "{body}"
    );
    assert!(
        body.contains("(shape not_fk) column owner_id must not be a foreign key"),
        "{body}"
    );
    assert!(!body.contains("(shape quoted)"), "{body}");
    assert!(
        body.contains("(shape other) foreign key on owner_id must reference"),
        "{body}"
    );
    assert!(
        body.contains("no column matches ^absent$ with nullable"),
        "{body}"
    );
}

#[test]
fn custom_message_and_include_filter_the_catalog() {
    let mut configured = config(
        "schemaCatalogPath: edges.json\nshapes:\n  - name: empty\n    tablePattern: ^empty_key$\n    primaryKeyTypes: [uuid]\n",
    );
    configured.rules[0].message = Some("use a real key".to_string());
    let root = fixture();
    let messages = check_with_files(&root, &configured, &[root.join("edges.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec!["edges.json: table:empty_key: use a real key".to_string()]
    );
    configured.rules[0].include = vec!["missing.json".to_string()];
    assert!(
        check_with_files(&root, &configured, &[root.join("edges.json")])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn pattern_diagnostics_name_every_constraint() {
    let options = "schemaCatalogPath: edges.json
shapes:
  - name: typed
    tablePattern: ^partial$
    requiredColumns:
      - namePattern: ^note$
        type: uuid
  - name: required
    tablePattern: ^partial$
    requiredColumns:
      - namePattern: ^note$
        nullable: false
  - name: plain
    tablePattern: ^partial$
    requiredColumns:
      - namePattern: ^owner_id$
        foreignKey: false
  - name: action
    tablePattern: ^partial$
    requiredColumns:
      - namePattern: ^absent$
        onDelete: cascade
  - name: labels
    tablePattern: ^empty_key$
    requiredTriggers:
      - function: '\"a\"\"b\"'
        timing: after
        events: [insert]
      - function: fn_trunc
        events: [truncate]
        forEachRow: false
";
    let root = fixture();
    let body = check_with_files(&root, &config(options), &[root.join("edges.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        body.contains("no column matches ^note$ with type uuid"),
        "{body}"
    );
    assert!(
        body.contains("no column matches ^note$ with NOT NULL"),
        "{body}"
    );
    assert!(
        body.contains("no column matches ^owner_id$ with not a foreign key"),
        "{body}"
    );
    assert!(
        body.contains("no column matches ^absent$ with a foreign key, ON DELETE CASCADE"),
        "{body}"
    );
    assert!(
        body.contains("no AFTER INSERT FOR EACH ROW trigger executing a\"b()"),
        "{body}"
    );
    assert!(
        body.contains("no BEFORE TRUNCATE FOR EACH STATEMENT trigger executing fn_trunc()"),
        "{body}"
    );
}
