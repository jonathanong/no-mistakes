use super::support::{expect, expect_none, index};

fn plural() -> String {
    "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n  irregularPlurals: {person: people, child: children}\n  uncountable: [data, metadata, feedback, media]\n  nonPluralTokens: [status, analysis, sms, news, series]\n".to_string()
}

#[test]
fn valid_index_names() {
    let yaml = "\
schemaCatalogPath: schema.json\n\
patterns:\n  index: '^idx_{table}__[a-z0-9_]+$'\n  uniqueIndex: '^(idx|uq)_{table}__[a-z0-9_]+$'\n\
abbreviations:\n  enabled: true\n";
    expect_none(yaml, index("orders", "idx_orders__status", false, false));
    expect_none(
        yaml,
        serde_json::json!({"tables": {"orders": {"indexes": {
            "uq_orders__number": {"unique": true},
            "idx_orders__number": {"unique": true},
            "orders_pkey": {"unique": true, "primary": true, "constraintBacked": true}
        }}}}),
    );
    expect_none(yaml, index("orders", "idx_ordrs__status", false, false));
    expect_none(
        yaml,
        index(
            "order_line_items",
            "idx_ord_line_itms__order_id",
            false,
            false,
        ),
    );
    expect_none(
        yaml,
        index(
            "customer_invoice_lines",
            "idx_cust_inv_lines__invoice_id",
            false,
            false,
        ),
    );
    expect_none(
        yaml,
        index(
            "customer_subscription_renewal_reminder_delivery_attempts",
            "idx_cstmr_sbscrptn_rnwl_rmndr_dlvry_attmpts__account_id",
            false,
            false,
        ),
    );
    expect_none(
        yaml,
        index("order__archives", "idx_ord__arch__status", false, false),
    );
}

#[test]
fn valid_plural_names() {
    let yaml = plural();
    for name in [
        "account_people",
        "order_status_histories",
        "feedback",
        "data_events",
    ] {
        expect_none(&yaml, serde_json::json!({"tables": {name: {}}}));
    }
    expect_none(
        &format!("{yaml}  objects: [table, enum]\n"),
        serde_json::json!({"enums": {
            "invoice_statuses": {"values": ["open"]},
            "order_line_item_types": {"values": ["goods"]}
        }}),
    );
    expect_none(
        &yaml,
        serde_json::json!({"enums": {"invoice_status": {"values": ["open"]}}}),
    );
}

#[test]
fn valid_min_words_allow_and_link_names() {
    expect_none(
        "schemaCatalogPath: schema.json\ntableMinWords: 2\nallow:\n  - {object: 'table:accounts', reason: core entity}\n",
        serde_json::json!({"tables": {"accounts": {}, "order_line_items": {}, "a__b": {}}}),
    );
    let yaml = "\
schemaCatalogPath: schema.json\n\
plural:\n  enabled: true\n  ignorePatterns: ['^link__']\n\
doubleUnderscore:\n  allowPattern: '^link__[a-z0-9]+(_[a-z0-9]+)*__[a-z0-9_]+__[a-z0-9_]+$'\n";
    expect_none(
        yaml,
        serde_json::json!({"tables": {
            "link__accounts__follows__accounts": {},
            "link__account__follow__account": {}
        }}),
    );
}

#[test]
fn valid_column_and_display_name_mismatch() {
    let yaml = "schemaCatalogPath: schema.json\npatterns:\n  column: '^[a-z][a-z0-9_]*$'\n";
    expect_none(
        yaml,
        serde_json::json!({"tables": {"accounts": {"columns": {"display_name": {"dataType": "text"}}}}}),
    );
    expect(
        yaml,
        serde_json::json!({"tables": {"accounts": {"columns": {"displayName": {"dataType": "text"}}}}}),
        "schema.json: column:accounts.displayName: column name does not match pattern ^[a-z][a-z0-9_]*$",
    );
}
