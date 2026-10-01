use super::support::{expect, expect_none, findings, index, table, trigger_fn, INDEX};

#[test]
fn column_denied_token_does_not_repeat_on_the_table() {
    expect(
        "schemaCatalogPath: schema.json\ndeniedTokens:\n  - {token: cfg, replacement: configuration}\n",
        serde_json::json!({"tables": {"orders": {"columns": {"cfg_id": {"type": "text"}}}}}),
        "schema.json: column:orders.cfg_id: name uses denied token \"cfg\"; use \"configuration\"",
    );
}

#[test]
fn function_overloads_are_reported_once_per_snapshot_key() {
    let definition = "CREATE FUNCTION fn_cfg() RETURNS void LANGUAGE sql AS $$ SELECT 1; $$";
    let messages = super::support::messages(
        "schemaCatalogPath: schema.json\npatterns:\n  function: '^fn_[a-z]+$'\ndeniedTokens:\n  - {token: cfg, replacement: configuration}\n",
        serde_json::json!({"functions": {
            "fn_cfg()": {"definition": definition},
            "fn_cfg(integer)": {"definition": definition}
        }}),
    );
    assert_eq!(
        messages,
        vec![
            "schema.json: function:fn_cfg(): name uses denied token \"cfg\"; use \"configuration\"",
            "schema.json: function:fn_cfg(integer): name uses denied token \"cfg\"; use \"configuration\"",
        ]
    );
}

#[test]
fn uppercase_tokens_match_case_insensitively() {
    expect(
        "schemaCatalogPath: schema.json\ndeniedTokens:\n  - {token: cfg, replacement: configuration}\n",
        serde_json::json!({"tables": {"orders": {"columns": {"App_CFG": {"type": "text"}}}}}),
        "schema.json: column:orders.App_CFG: name uses denied token \"cfg\"; use \"configuration\"",
    );
}

#[test]
fn empty_patterns_produce_no_pattern_findings() {
    expect_none(
        "schemaCatalogPath: schema.json\npatterns: {}\n",
        serde_json::json!({"tables": {"Orders": {"indexes": {"NotAnIndex": {}}}}}),
    );
}

#[test]
fn table_placeholder_comparison_is_ascii_case_insensitive() {
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^IDX_{table}__STATUS$'\n",
        index("orders", "IDX_ORDERS__STATUS", false, false),
    );
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_{table}__[a-z0-9_]+$'\nabbreviations:\n  enabled: true\n",
        index("cafés", "idx_CAFÉS__id", false, false),
        "schema.json: index:cafés.idx_CAFÉS__id: index name does not match pattern ^idx_{table}__[a-z0-9_]+$ ({table} = cafés or an abbreviation such as cfés)",
    );
}

#[test]
fn matching_index_skips_denied_tokens_inside_the_table_middle() {
    let yaml = format!("{INDEX}deniedTokens:\n  - {{token: cfg, replacement: configuration}}\n");
    let messages = super::support::messages(
        &yaml,
        serde_json::json!({"tables": {"cfg": {"indexes": {"idx_cfg__id": {}}}}}),
    );
    assert_eq!(
        messages,
        vec!["schema.json: table:cfg: name uses denied token \"cfg\"; use \"configuration\""]
    );
    let messages = super::support::messages(&yaml, index("orders", "idx_cfg__id", false, false));
    assert!(messages
        .iter()
        .any(|message| message.contains("index:orders.idx_cfg__id")));
    assert!(messages
        .iter()
        .any(|message| message.contains("denied token")));
}

#[test]
fn trigger_placeholder_matches_like_an_index() {
    let yaml = "\
schemaCatalogPath: schema.json\n\
patterns:\n  trigger: '^trg_{table}__[a-z0-9_]+$'\n\
abbreviations:\n  enabled: true\n";
    expect_none(
        yaml,
        serde_json::json!({"tables": {"orders": {"triggers": {"trg_ordrs__audit": {"definition": "CREATE TRIGGER trg_ordrs__audit AFTER INSERT ON orders FOR EACH ROW EXECUTE FUNCTION fn()"}}}}}),
    );
    expect(
        yaml,
        serde_json::json!({"tables": {"order_line_items": {"triggers": {"trg_oli__audit": {"definition": "CREATE TRIGGER trg_oli__audit AFTER INSERT ON order_line_items FOR EACH ROW EXECUTE FUNCTION fn()"}}}}}),
        "schema.json: trigger:order_line_items.trg_oli__audit: trigger name does not match pattern ^trg_{table}__[a-z0-9_]+$ ({table} = order_line_items or an abbreviation such as ordr_lin_itms)",
    );
}

#[test]
fn trigger_functions_patterns_are_independent() {
    let yaml = "\
schemaCatalogPath: schema.json\n\
patterns:\n  function: '^fn_[a-z0-9_]+$'\n  triggerFunction: '^fn_(reject|update|project|create|lock)_[a-z0-9_]+$'\n";
    expect_none(yaml, trigger_fn("fn_lock_orders", "trigger"));
    expect(
        yaml,
        trigger_fn("fn_guard_orders", "trigger"),
        "schema.json: function:fn_guard_orders: trigger function name does not match pattern ^fn_(reject|update|project|create|lock)_[a-z0-9_]+$",
    );
    let messages = super::support::messages(yaml, trigger_fn("guard_orders", "trigger"));
    assert_eq!(messages.len(), 2, "{messages:#?}");
    assert!(messages
        .iter()
        .any(|message| message.contains(": function name does not match")));
    assert!(messages
        .iter()
        .any(|message| message.contains("trigger function name does not match")));
    expect(
        yaml,
        trigger_fn("guard_orders", "integer"),
        "schema.json: function:guard_orders: function name does not match pattern ^fn_[a-z0-9_]+$",
    );
}

#[test]
fn views_enums_and_partitioned_tables_use_their_kinds() {
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  view: '^view_[a-z0-9_]+$'\n  materializedView: '^mv_[a-z0-9_]+$'\n",
        serde_json::json!({"views": {"stats": {"definition": "select 1", "materialized": true}}}),
        "schema.json: materialized-view:stats: materialized view name does not match pattern ^mv_[a-z0-9_]+$",
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  materializedView: '^mv_[a-z0-9_]+$'\n",
        serde_json::json!({"views": {"mv_stats": {"definition": "select 1", "materialized": true}}}),
    );
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  view: '^view_[a-z0-9_]+$'\nplural:\n  enabled: true\ntableMinWords: 2\n",
        serde_json::json!({"views": {"widgets": {"definition": "select 1"}}}),
        "schema.json: view:widgets: view name does not match pattern ^view_[a-z0-9_]+$",
    );
    let enum_only = super::support::messages(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n  objects: [enum]\ntableMinWords: 2\n",
        table("invoice"),
    );
    assert_eq!(
        enum_only,
        vec!["schema.json: table:invoice: table name has 1 word; use at least 2 that name the owner and the thing (for example <owner>_invoice)"]
    );
    let partitioned = super::support::messages(
        "schemaCatalogPath: schema.json\npatterns:\n  table: '^[a-z_]+$'\nplural:\n  enabled: true\ntableMinWords: 2\n",
        serde_json::json!({"tables": {"widget": {"relationKind": "partitioned table"}}}),
    );
    assert_eq!(
        partitioned,
        vec![
            "schema.json: table:widget: table name has 1 word; use at least 2 that name the owner and the thing (for example <owner>_widget)",
            "schema.json: table:widget: table name must end in a plural word; \"widget\" is singular",
        ]
    );
}

#[test]
fn double_underscore_without_a_pattern_reports_every_separator() {
    for (kind, body, object) in [
        (
            "tables",
            serde_json::json!({"tables": {"orders__archive": {}}}),
            "table:orders__archive",
        ),
        (
            "views",
            serde_json::json!({"views": {"orders__archive": {"definition": "select 1"}}}),
            "view:orders__archive",
        ),
        (
            "enums",
            serde_json::json!({"enums": {"orders__archive": {"values": ["a"]}}}),
            "enum:orders__archive",
        ),
    ] {
        let _ = kind;
        expect(
            "schemaCatalogPath: schema.json\ndoubleUnderscore: {}\n",
            body,
            &format!("schema.json: {object}: \"__\" is reserved"),
        );
    }
}

#[test]
fn plural_ignore_and_non_final_irregular_values() {
    let yaml = "\
schemaCatalogPath: schema.json\n\
plural:\n  enabled: true\n  ignorePatterns: ['^link__']\n  irregularPlurals: {person: people}\n";
    expect_none(yaml, table("link__account"));
    expect(
        yaml,
        table("account"),
        "schema.json: table:account: table name must end in a plural word; \"account\" is singular",
    );
    expect(
        yaml,
        table("people_accounts"),
        "schema.json: table:people_accounts: only the last word of a table name is plural; \"people\" is plural",
    );
    expect(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n  nonPluralTokens: [sms, news, series]\n",
        table("inbox_sms"),
        "schema.json: table:inbox_sms: table name must end in a plural word; \"sms\" is singular",
    );
}

#[test]
fn unique_index_uses_index_pattern_when_unique_index_is_unset() {
    expect(
        INDEX,
        index("orders", "uq_orders__id", true, false),
        "schema.json: index:orders.uq_orders__id: index name does not match pattern ^idx_{table}__[a-z0-9_]+$ ({table} = orders or an abbreviation such as ordrs)",
    );
}

#[test]
fn unanchored_pattern_matches_a_substring() {
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: idx_\n",
        index("orders", "xidx_orders", false, false),
    );
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_'\n",
        index("orders", "xidx_orders", false, false),
        "schema.json: index:orders.xidx_orders: index name does not match pattern ^idx_",
    );
}

#[test]
fn allow_suppresses_a_finding_and_reports_stale_entries() {
    let yaml = "\
schemaCatalogPath: schema.json\n\
tableMinWords: 2\n\
allow:\n  - {object: 'table:widgets', reason: core}\n  - {object: 'table:retired', reason: gone}\n";
    let found = findings(yaml, table("widgets"));
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].rule, "postgres-object-naming");
    assert_eq!(found[0].file, "schema.json");
    assert_eq!(found[0].line, 1);
    assert_eq!(found[0].target.as_deref(), Some("table:retired"));
    assert_eq!(
        found[0].message,
        "schema.json: stale postgres-object-naming allow entry: table:retired"
    );
    let again = findings(yaml, table("widgets"));
    assert_eq!(found, again);
}

#[test]
fn loose_pattern_uses_the_longest_middle_for_tokens() {
    let yaml = "\
schemaCatalogPath: schema.json\n\
patterns:\n  index: '^idx_{table}[a-z0-9_]*$'\n\
abbreviations:\n  enabled: true\n\
deniedTokens:\n  - {token: orders, replacement: purchases}\n";
    expect(
        yaml,
        index("orders", "idx_orders__x", false, false),
        "schema.json: table:orders: name uses denied token \"orders\"; use \"purchases\"",
    );
}
