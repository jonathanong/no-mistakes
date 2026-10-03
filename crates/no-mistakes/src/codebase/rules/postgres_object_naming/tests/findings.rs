use super::support::{expect, expect_none, index, table, trigger_fn, INDEX};

const IDX: &str = "index name does not match pattern ^idx_{table}__[a-z0-9_]+$";
const ORDERS: &str = "({table} = orders or an abbreviation such as ordrs)";
const LINES: &str = "({table} = order_line_items or an abbreviation such as ordr_lin_itms)";

#[test]
fn finding_text_table() {
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  function: '^fn_[a-z0-9_]+$'\n  triggerFunction: '^fn_(reject|update|project|create|lock)_[a-z0-9_]+$'\n",
        trigger_fn("fn_guard_orders_total", "trigger"),
        "schema.json: function:fn_guard_orders_total: trigger function name does not match pattern ^fn_(reject|update|project|create|lock)_[a-z0-9_]+$",
    );
    expect(
        INDEX,
        index("orders", "orders_status_ix", false, false),
        &format!("schema.json: index:orders.orders_status_ix: {IDX} {ORDERS}"),
    );
    let both = "\
schemaCatalogPath: schema.json\n\
patterns:\n  index: '^idx_{table}__[a-z0-9_]+$'\n  uniqueIndex: '^(idx|uq)_{table}__[a-z0-9_]+$'\n\
abbreviations:\n  enabled: true\n";
    expect(
        both,
        index("orders", "uq_orders__status", false, false),
        &format!("schema.json: index:orders.uq_orders__status: {IDX} {ORDERS}"),
    );
    let line = "index name does not match pattern ^idx_{table}__[a-z0-9_]+$";
    for name in [
        "idx_oli__order_id",
        "idx_order_items__order_id",
        "idx_ord_ln_itms__order_id",
    ] {
        expect(
            INDEX,
            index("order_line_items", name, false, false),
            &format!("schema.json: index:order_line_items.{name}: {line} {LINES}"),
        );
    }
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_{table}__[a-z0-9_]+$'\n",
        index("orders", "idx_ordrs__status", false, false),
        "schema.json: index:orders.idx_ordrs__status: index name does not match pattern ^idx_{table}__[a-z0-9_]+$ ({table} = orders)",
    );
    expect(
        "schemaCatalogPath: schema.json\ndeniedTokens:\n  - {token: cfg, replacement: configuration}\n",
        table("app_cfg_values"),
        "schema.json: table:app_cfg_values: name uses denied token \"cfg\"; use \"configuration\"",
    );
    expect(
        &format!("{INDEX}deniedTokens:\n  - {{token: cfg, replacement: configuration}}\n"),
        index("orders", "idx_orders__cfg_id", false, false),
        "schema.json: index:orders.idx_orders__cfg_id: name uses denied token \"cfg\"; use \"configuration\"",
    );
    expect(
        "schemaCatalogPath: schema.json\nspelling:\n  acknowledgement: acknowledgment\n",
        serde_json::json!({"tables": {"orders": {"columns": {"acknowledgement_at": {"dataType": "timestamptz"}}}}}),
        "schema.json: column:orders.acknowledgement_at: name spells \"acknowledgement\"; use \"acknowledgment\"",
    );
    expect(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n",
        table("invoice_line"),
        "schema.json: table:invoice_line: table name must end in a plural word; \"line\" is singular",
    );
    expect(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n  objects: [table, enum]\n",
        serde_json::json!({"enums": {"invoice_status": {"values": ["open"]}}}),
        "schema.json: enum:invoice_status: enum name must end in a plural word; \"status\" is singular",
    );
    expect(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n",
        table("user_roles_grants"),
        "schema.json: table:user_roles_grants: only the last word of a table name is plural; \"roles\" is plural",
    );
    expect(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n  irregularPlurals: {person: people}\n",
        table("account_person"),
        "schema.json: table:account_person: table name must end in a plural word; \"person\" is singular (plural: \"people\")",
    );
    expect(
        "schemaCatalogPath: schema.json\ntableMinWords: 2\n",
        table("widgets"),
        "schema.json: table:widgets: table name has 1 word; use at least 2 that name the owner and the thing (for example owner_widgets)",
    );
    let pattern = "^link__[a-z0-9]+(_[a-z0-9]+)*__[a-z0-9_]+__[a-z0-9_]+$";
    expect(
        &format!(
            "schemaCatalogPath: schema.json\ndoubleUnderscore:\n  allowPattern: '{pattern}'\n"
        ),
        table("orders__archive"),
        &format!(
            "schema.json: table:orders__archive: \"__\" is reserved for names matching {pattern}"
        ),
    );
}

#[test]
fn sky_omits_the_suggestion_clause() {
    expect(
        INDEX,
        index("sky", "sky_ix", false, false),
        "schema.json: index:sky.sky_ix: index name does not match pattern ^idx_{table}__[a-z0-9_]+$ ({table} = sky)",
    );
}

#[test]
fn constraint_backed_indexes_are_skipped_unless_configured() {
    expect_none(INDEX, index("orders", "orders_pkey", true, true));
    expect(
        &format!("{INDEX}checkConstraintBackedIndexes: true\n"),
        index("orders", "orders_pkey", true, true),
        &format!("schema.json: index:orders.orders_pkey: {IDX} {ORDERS}"),
    );
}
