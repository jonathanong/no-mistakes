use super::support::{at, column, expect, expect_none, TYPES};
use std::collections::BTreeMap;

const BOOL_HINT: &str = "name boolean columns as a predicate: is_, has_, can_ or should_";
const FORBIDDEN: &str = "column name matches forbidden pattern (^|_)table(_name)?$; a column must not name a table; use one table per target, or an enum";

#[test]
fn type_and_name_rules_use_the_configured_text() {
    expect(
        TYPES,
        column("invoices", "due", "timestamp with time zone"),
        &at(
            "invoices",
            "due",
            "timestamp with time zone column name does not match _at$ (end timestamp columns in _at)",
        ),
    );
    expect(
        TYPES,
        column("accounts", "active", "boolean"),
        &at(
            "accounts",
            "active",
            &format!("boolean column name does not match (^|_)(is|has|can|should)_ ({BOOL_HINT})"),
        ),
    );
    expect(
        TYPES,
        column("subscriptions", "since", "date"),
        &at(
            "subscriptions",
            "since",
            "date column name does not match (^day|_on)$ (name date columns day or end them in _on)",
        ),
    );
    let paid = "column name matches _at$ but its type is date; use timestamp with time zone (a column ending in _at holds a timestamp with time zone)";
    let date =
        "date column name does not match (^day|_on)$ (name date columns day or end them in _on)";
    assert_eq!(
        super::support::messages(TYPES, column("orders", "shipped_at", "date")),
        vec![
            at("orders", "shipped_at", paid),
            at("orders", "shipped_at", date)
        ]
    );
    expect(
        TYPES,
        column("invoices", "paid_at", "timestamp without time zone"),
        &at(
            "invoices",
            "paid_at",
            "column name matches _at$ but its type is timestamp without time zone; use timestamp with time zone (a column ending in _at holds a timestamp with time zone)",
        ),
    );
    expect(
        TYPES,
        column("sync_jobs", "cursor_order_id", "text"),
        &at(
            "sync_jobs",
            "cursor_order_id",
            "column name matches ^cursor_.*_id$ but its type is text; use uuid (a keyset cursor holds the swept table's uuid primary key)",
        ),
    );
}

#[test]
fn valid_type_names_and_array_types_pass() {
    for (table, name, data_type) in [
        ("invoices", "due_at", "timestamp with time zone"),
        ("accounts", "is_active", "boolean"),
        ("votes", "score_is_neutral", "boolean"),
        ("invoices", "issued_on", "date"),
        ("daily_totals", "day", "date"),
        ("sync_jobs", "cursor_order_id", "uuid"),
        ("flags", "active", "boolean[]"),
        ("ids", "thing_id", "uuid[]"),
    ] {
        expect_none(TYPES, column(table, name, data_type));
    }
}

#[test]
fn type_comparison_is_case_insensitive_and_keeps_the_stored_type() {
    expect(
        "schemaCatalogPath: schema.json\ntypeRules:\n  - types: [boolean]\n    namePattern: '^is_'\n",
        column("accounts", "active", "BOOLEAN"),
        &at("accounts", "active", "BOOLEAN column name does not match ^is_"),
    );
}

#[test]
fn a_missing_hint_is_omitted_and_every_matching_rule_reports() {
    expect(
        "schemaCatalogPath: schema.json\ntypeRules:\n  - types: ['timestamp with time zone']\n    namePattern: _at$\n",
        column("invoices", "due", "timestamp with time zone"),
        &at(
            "invoices",
            "due",
            "timestamp with time zone column name does not match _at$",
        ),
    );
    expect(
        "schemaCatalogPath: schema.json\nnameTypeRules:\n  - namePattern: _at$\n    types: ['timestamp with time zone']\n",
        column("invoices", "paid_at", "date"),
        &at(
            "invoices",
            "paid_at",
            "column name matches _at$ but its type is date; use timestamp with time zone",
        ),
    );
    let yaml = "schemaCatalogPath: schema.json\ntypeRules:\n  - types: [boolean]\n    namePattern: '^is_'\n    hint: one\n  - types: [boolean]\n    namePattern: _active$\n    hint: two\n";
    assert_eq!(
        super::support::messages(yaml, column("accounts", "flag", "boolean")),
        vec![
            at(
                "accounts",
                "flag",
                "boolean column name does not match ^is_ (one)"
            ),
            at(
                "accounts",
                "flag",
                "boolean column name does not match _active$ (two)"
            ),
        ]
    );
    let names = "schemaCatalogPath: schema.json\nnameTypeRules:\n  - namePattern: _id$\n    types: [uuid]\n    hint: one\n  - namePattern: '^cursor_'\n    types: [text]\n    hint: two\n";
    assert_eq!(
        super::support::messages(names, column("sync", "cursor_order_id", "integer")),
        vec![
            at(
                "sync",
                "cursor_order_id",
                "column name matches ^cursor_ but its type is integer; use text (two)"
            ),
            at(
                "sync",
                "cursor_order_id",
                "column name matches _id$ but its type is integer; use uuid (one)"
            ),
        ]
    );
}

#[test]
fn generated_columns_follow_skip_generated_columns() {
    let generated = serde_json::json!({
        "tables": { "entities": { "columns": { "created_at": { "dataType": "date", "generated": "stored" } } } }
    });
    expect_none(
        "schemaCatalogPath: schema.json\nskipGeneratedColumns: true\ntypeRules:\n  - types: [date]\n    namePattern: _on$\nnameTypeRules:\n  - namePattern: _at$\n    types: ['timestamp with time zone']\n",
        generated.clone(),
    );
    assert_eq!(
        super::support::messages(
            "schemaCatalogPath: schema.json\nskipGeneratedColumns: false\ntypeRules:\n  - types: [date]\n    namePattern: _on$\n    hint: day\nnameTypeRules:\n  - namePattern: _at$\n    types: ['timestamp with time zone']\n    hint: time\n",
            generated,
        )
        .len(),
        2
    );
}

#[test]
fn allow_suppresses_every_finding_for_the_column() {
    let yaml = "schemaCatalogPath: schema.json\ntypeRules:\n  - types: [date]\n    namePattern: _on$\nnameTypeRules:\n  - namePattern: _at$\n    types: ['timestamp with time zone']\nallow:\n  - object: 'column:orders.shipped_at'\n    reason: protocol\n";
    expect_none(yaml, column("orders", "shipped_at", "date"));
}

#[test]
fn forbidden_names_report_the_first_match_only() {
    let yaml = "schemaCatalogPath: schema.json\nforbiddenColumnNames:\n  - pattern: '(^|_)table(_name)?$'\n    hint: 'a column must not name a table; use one table per target, or an enum'\n";
    expect(
        yaml,
        column("comments", "target_table", "text"),
        &at("comments", "target_table", FORBIDDEN),
    );
    expect(
        yaml,
        column("comments", "table_name", "target_kind"),
        &at("comments", "table_name", FORBIDDEN),
    );
    expect(
        yaml,
        column("comments", "target_table", "integer"),
        &at("comments", "target_table", FORBIDDEN),
    );
    expect_none(yaml, column("schedules", "timetable", "text"));
    let first = "schemaCatalogPath: schema.json\nforbiddenColumnNames:\n  - pattern: table\n    hint: first\n  - pattern: '_table$'\n    hint: second\n";
    expect(
        first,
        column("comments", "target_table", "text"),
        &at(
            "comments",
            "target_table",
            "column name matches forbidden pattern table; first",
        ),
    );
}

#[test]
fn singularizer_applies_overrides_then_suffix_rules() {
    let mut overrides = BTreeMap::new();
    overrides.insert("people".to_string(), "person".to_string());
    overrides.insert("aliases".to_string(), "alias".to_string());
    overrides.insert("statuses".to_string(), "status".to_string());
    let singular = |token: &str| super::super::singular::singularize(token, &overrides);
    assert_eq!(singular("items"), "item");
    assert_eq!(singular("currencies"), "currency");
    assert_eq!(singular("addresses"), "address");
    assert_eq!(singular("boxes"), "box");
    assert_eq!(singular("batches"), "batch");
    assert_eq!(singular("dishes"), "dish");
    assert_eq!(singular("staff"), "staff");
    assert_eq!(singular("people"), "person");
    assert_eq!(singular("People"), "person");
    assert_eq!(singular("aliases"), "alias");
    assert_eq!(singular("statuses"), "status");
    assert_eq!(
        super::super::singular::singular_name("order_line_items", &overrides),
        "order_line_item"
    );
    assert_eq!(
        super::super::singular::singular_name("foo__bars", &overrides),
        "foo__bar"
    );
}
