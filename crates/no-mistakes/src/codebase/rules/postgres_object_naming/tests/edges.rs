use super::support::{expect, expect_err, expect_none, findings, index, table, trigger_fn, INDEX};

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
fn qualified_names_use_the_unqualified_identifier() {
    let yaml = "\
schemaCatalogPath: schema.json\n\
patterns:\n  table: '^[a-z][a-z0-9_]*$'\n  index: '^idx_{table}__[a-z0-9_]+$'\n\
allow:\n  - {object: 'table:public.order_items', reason: kept}\n";
    expect(
        yaml,
        serde_json::json!({"tables": {"public.order_items": {"indexes": {"idx_order_items__id": {}}}}}),
        "schema.json: stale postgres-object-naming allow entry: table:public.order_items",
    );
}

#[test]
fn inline_flag_applies_to_both_sides_of_the_table_placeholder() {
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^(?i)idx_{table}__[a-z]+$'\n",
        index("orders", "idx_orders__ID", false, false),
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '(?i)^idx_{table}__[a-z]+$'\n",
        index("orders", "IDX_orders__id", false, false),
    );
}

#[test]
fn abbreviation_only_denied_tokens_stay_visible() {
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_{table}__[a-z0-9_]+$'\nabbreviations:\n  enabled: true\ndeniedTokens:\n  - {token: cfg, replacement: configuration}\n",
        index("configuration", "idx_cfg__id", false, false),
        "schema.json: index:configuration.idx_cfg__id: name uses denied token \"cfg\"; use \"configuration\"",
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
fn event_triggers_use_the_trigger_function_pattern() {
    let definition = "CREATE FUNCTION fn_guard() RETURNS event_trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END; $$";
    let yaml = "\
schemaCatalogPath: schema.json\n\
patterns:\n  function: '^fn_[a-z0-9_]+$'\n  triggerFunction: '^fn_(reject|update|project|create|lock)_[a-z0-9_]+$'\n";
    expect(
        yaml,
        serde_json::json!({"functions": {"fn_guard()": {"definition": definition}}}),
        "schema.json: function:fn_guard(): trigger function name does not match pattern ^fn_(reject|update|project|create|lock)_[a-z0-9_]+$",
    );
    expect_none(yaml, trigger_fn("fn_lock_orders", "event_trigger"));
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
        vec!["schema.json: table:invoice: table name has 1 word; use at least 2 that name the owner and the thing (for example owner_invoice)"]
    );
    let partitioned = super::support::messages(
        "schemaCatalogPath: schema.json\npatterns:\n  table: '^[a-z_]+$'\nplural:\n  enabled: true\ntableMinWords: 2\n",
        serde_json::json!({"tables": {"widget": {"relationKind": "partitioned table"}}}),
    );
    assert_eq!(
        partitioned,
        vec![
            "schema.json: table:widget: table name has 1 word; use at least 2 that name the owner and the thing (for example owner_widget)",
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
fn quoted_names_min_words_and_active_flags() {
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  table: '^order_items$'\n",
        table("public.\"order_items\""),
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  table: '^a\"b$'\n",
        table("public.\"a\"\"b\""),
    );
    expect_none(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n",
        table("___"),
    );
    expect(
        "schemaCatalogPath: schema.json\ntableMinWords: 3\n",
        table("widgets"),
        "schema.json: table:widgets: table name has 1 word; use at least 3 that name the owner and the thing (for example owner_part_widgets)",
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_(?i){table}__[a-z]+$'\n",
        index("orders", "idx_orders__ID", false, false),
    );
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '(?i)(?-i)^idx_{table}__[a-z]+$'\n",
        index("orders", "IDX_orders__id", false, false),
        "schema.json: index:orders.IDX_orders__id: index name does not match pattern (?i)(?-i)^idx_{table}__[a-z]+$ ({table} = orders)",
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^.*{table}.*$'\n",
        index("ab", "abab", false, false),
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_{table}_[^]]x$'\n",
        index("orders", "idx_orders_bx", false, false),
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_{table}_\\$x$'\n",
        index("orders", "idx_orders_$x", false, false),
    );
}

#[test]
fn scoped_verbose_comments_hide_placeholders() {
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: \"^(?x:idx_ # {table}\\n){table}__#$\"\n",
        index("orders", "idx_orders__#", false, false),
    );
    expect_err(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^(?x:idx_{table})__$'\n",
        "option patterns.index: {table} must not be inside a group, a character class or an alternation",
    );
}

#[test]
fn verbose_anchors_and_quoted_function_names() {
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: \"(?x)^idx_{table}__[a-z]+$ # naming\"\n",
        index("orders", "idx_orders__id", false, false),
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: \"(?x) # note\\n^idx_{table}__[a-z]+$\"\n",
        index("orders", "idx_orders__id", false, false),
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  function: '^fn\\(foo$'\n",
        serde_json::json!({
            "functions": {
                "\"fn(foo\"()": {
                    "definition": "CREATE FUNCTION \"fn(foo\"() RETURNS void LANGUAGE sql AS $$ SELECT 1; $$"
                }
            }
        }),
    );
}

#[test]
fn verbose_flag_comments_stay_literal() {
    let yaml = "schemaCatalogPath: schema.json\npatterns:\n  index: \"(?x)^idx_ # (?i)\\n{table}__[a-z]+$\"\n";
    expect_none(yaml, index("orders", "idx_orders__id", false, false));
    expect(
        yaml,
        index("orders", "idx_orders__ID", false, false),
        "schema.json: index:orders.idx_orders__ID: index name does not match pattern (?x)^idx_ # (?i)\n{table}__[a-z]+$ ({table} = orders)",
    );
}

#[test]
fn verbose_comments_and_multiline_anchors() {
    let verbose =
        "schemaCatalogPath: schema.json\npatterns:\n  index: \"(?x)^idx_ # {table}\\n[a-z]+$\"\n";
    expect_none(verbose, index("orders", "idx_ab", false, false));
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '(?m)^idx_{table}__x$'\n",
        index("orders", "idx_\nJUNKorders__x", false, false),
        "schema.json: index:orders.idx_\nJUNKorders__x: index name does not match pattern (?m)^idx_{table}__x$ ({table} = orders)",
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '(?m)^idx_{table}__x$'\n",
        index("orders", "idx_orders__x", false, false),
    );
}

#[test]
fn primary_indexes_and_flags_inside_classes() {
    let primary = serde_json::json!({
        "tables": {
            "orders": {
                "indexes": {
                    "orders_pkey": { "unique": true, "primary": true }
                }
            }
        }
    });
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_'\n",
        primary.clone(),
    );
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^idx_'\ncheckConstraintBackedIndexes: true\n",
        primary,
        "schema.json: index:orders.orders_pkey: index name does not match pattern ^idx_",
    );
    expect_none(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^[x(?i)]{table}__[a-z]+$'\n",
        index("orders", "xorders__id", false, false),
    );
    expect(
        "schemaCatalogPath: schema.json\npatterns:\n  index: '^[x(?i)]{table}__[a-z]+$'\n",
        index("orders", "xorders__ID", false, false),
        "schema.json: index:orders.xorders__ID: index name does not match pattern ^[x(?i)]{table}__[a-z]+$ ({table} = orders)",
    );
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
