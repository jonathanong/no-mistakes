use super::support::{column, expect, expect_none, messages, PATH};

const ORDINARY: &str = "column is an array (text[]); store the values as child rows or an enum, or add an allow entry with a reason";
const NEVER: &str = "column is a uuid[] array; model each reference as a child row with a foreign key (uuid arrays cannot be allowlisted)";

#[test]
fn text_arrays_are_ordinary_and_uuid_arrays_cannot_be_excused() {
    expect(
        PATH,
        column("orders", "tag_names", "text[]"),
        &format!("column:orders.tag_names: {ORDINARY}"),
    );
    let yaml = "\
schemaCatalogPath: schema.json
allow:
  - object: column:orders.coupon_ids
    reason: tried to keep the array
";
    let body = serde_json::json!({
        "tables": {
            "orders": {
                "columns": {
                    "tag_names": { "dataType": "text[]" },
                    "coupon_ids": { "dataType": "uuid[]" }
                }
            }
        }
    });
    let messages = messages(yaml, body);
    assert!(
        messages
            .iter()
            .any(|message| message.contains("column:orders.tag_names: ")
                && message.contains(ORDINARY)),
        "{messages:#?}"
    );
    assert!(
        messages.iter().any(
            |message| message.contains("column:orders.coupon_ids: ") && message.contains(NEVER)
        ),
        "{messages:#?}"
    );
    assert!(
        messages.iter().any(|message| {
            message
                == "schema.json: column:orders.coupon_ids: allow entry column:orders.coupon_ids cannot excuse a uuid[] column"
        }),
        "{messages:#?}"
    );
    assert!(
        messages.iter().all(|message| !message.contains("stale")),
        "{messages:#?}"
    );
    assert_eq!(messages.len(), 3, "{messages:#?}");
}

#[test]
fn allow_enum_and_element_types_skip_arrays() {
    let oauth = "\
schemaCatalogPath: schema.json
allow:
  - object: column:oauth_clients.redirect_uris
    reason: OAuth 2.0 client metadata (RFC 7591) defines this as a list
";
    expect_none(oauth, column("oauth_clients", "redirect_uris", "text[]"));
    let weekdays = serde_json::json!({
        "enums": { "Weekday": { "values": ["mon", "tue"] } },
        "tables": { "shipments": { "columns": { "weekdays": { "dataType": "weekday[]" } } } }
    });
    let qualified = serde_json::json!({
        "enums": { "public.report_types": { "values": ["a"] } },
        "tables": { "reports": { "columns": { "kinds": { "dataType": "report_types[]" } } } }
    });
    expect_none(
        "schemaCatalogPath: schema.json\nallowEnumElements: true\n",
        qualified,
    );
    expect_none(
        "schemaCatalogPath: schema.json\nallowEnumElements: true\n",
        weekdays.clone(),
    );
    expect(
        PATH,
        weekdays,
        "column:shipments.weekdays: column is an array (weekday[]); store the values as child rows or an enum, or add an allow entry with a reason",
    );
    expect_none(
        "schemaCatalogPath: schema.json\nallowElementTypes: [smallint]\n",
        column("sensors", "samples", "smallint[]"),
    );
}

#[test]
fn multidimensional_and_spaced_element_types() {
    expect_none(
        "schemaCatalogPath: schema.json\nallowElementTypes: [integer]\n",
        column("metrics", "buckets", "integer[][]"),
    );
    expect(
        PATH,
        column("metrics", "buckets", "integer[][]"),
        "column:metrics.buckets: column is an array (integer[][]); store the values as child rows or an enum, or add an allow entry with a reason",
    );
    expect(
        PATH,
        column("labels", "names", "character varying[]"),
        "column:labels.names: column is an array (character varying[]); store the values as child rows or an enum, or add an allow entry with a reason",
    );
    expect_none(
        "schemaCatalogPath: schema.json\nallowElementTypes: ['character varying']\n",
        column("labels", "names", "character varying[]"),
    );
    expect(
        PATH,
        column("orders", "coupon_ids", "Uuid[]"),
        "column:orders.coupon_ids: column is a Uuid[] array; model each reference as a child row with a foreign key (Uuid arrays cannot be allowlisted)",
    );
    expect(
        PATH,
        column("orders", "coupon_ids", "uuid[][]"),
        "column:orders.coupon_ids: column is a uuid[] array; model each reference as a child row with a foreign key (uuid arrays cannot be allowlisted)",
    );
    expect_none(PATH, column("orders", "note", "text"));
}

#[test]
fn omitted_never_allow_defaults_to_uuid_and_an_empty_list_does_not() {
    expect(
        PATH,
        column("orders", "coupon_ids", "uuid[]"),
        &format!("column:orders.coupon_ids: {NEVER}"),
    );
    let yaml = "\
schemaCatalogPath: schema.json
neverAllowElementTypes: []
allow:
  - object: column:orders.coupon_ids
    reason: this project stores uuid lists
";
    expect_none(yaml, column("orders", "coupon_ids", "uuid[]"));
}

#[test]
fn unused_allow_entries_are_stale() {
    let yaml = "\
schemaCatalogPath: schema.json
allow:
  - object: column:accounts.missing
    reason: leftover
";
    expect(
        yaml,
        column("accounts", "display_name", "text"),
        "stale postgres-array-columns allow entry: column:accounts.missing",
    );
}
