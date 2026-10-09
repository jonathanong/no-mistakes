use super::support::{catalog, expect, expect_none, messages, PATH};

const ORDINARY: &str = "column is an array (text[]); store the values as child rows or an enum, or add an allow entry with a reason";
const NEVER: &str = "column is a uuid[] array; model each reference as a child row with a foreign key (uuid arrays cannot be allowlisted)";

#[test]
fn text_arrays_are_ordinary_and_uuid_arrays_cannot_be_excused() {
    expect(
        PATH,
        catalog("orders-tag-names"),
        &format!("column:orders.tag_names: {ORDINARY}"),
    );
    let yaml = "\
schemaCatalogPath: schema.json
allow:
  - object: column:orders.coupon_ids
    reason: tried to keep the array
";
    let messages = messages(yaml, catalog("orders-tags-and-coupons"));
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
    expect_none(oauth, catalog("oauth-redirect-uris"));
    let weekdays = catalog("weekday-enum");
    expect_none(
        "schemaCatalogPath: schema.json\nallowEnumElements: true\n",
        catalog("report-types"),
    );
    expect(
        "schemaCatalogPath: schema.json\nallowEnumElements: true\n",
        catalog("audit-report-types"),
        "column:reports.kinds: column is an array (audit.report_types[]); store the values as child rows or an enum, or add an allow entry with a reason",
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
        catalog("sensor-samples"),
    );
}

#[test]
fn external_enum_names_do_not_alias_a_local_domain_array() {
    expect(
        "schemaCatalogPath: schema.json\nallowEnumElements: true\n",
        catalog("external-enum-domain-shadow"),
        "column:domain_arrays.values: column is an array (priority[]); store the values as child rows or an enum, or add an allow entry with a reason",
    );
}

#[test]
fn multidimensional_and_spaced_element_types() {
    expect_none(
        "schemaCatalogPath: schema.json\nallowElementTypes: [integer]\n",
        catalog("metric-buckets"),
    );
    expect(
        PATH,
        catalog("metric-buckets"),
        "column:metrics.buckets: column is an array (integer[][]); store the values as child rows or an enum, or add an allow entry with a reason",
    );
    expect(
        PATH,
        catalog("label-names"),
        "column:labels.names: column is an array (character varying[]); store the values as child rows or an enum, or add an allow entry with a reason",
    );
    expect_none(
        "schemaCatalogPath: schema.json\nallowElementTypes: ['character varying']\n",
        catalog("label-names"),
    );
    expect(
        PATH,
        catalog("coupon-ids-mixed-case"),
        "column:orders.coupon_ids: column is a Uuid[] array; model each reference as a child row with a foreign key (Uuid arrays cannot be allowlisted)",
    );
    expect(
        PATH,
        catalog("coupon-ids-nested"),
        "column:orders.coupon_ids: column is a uuid[] array; model each reference as a child row with a foreign key (uuid arrays cannot be allowlisted)",
    );
    expect_none(PATH, catalog("orders-note"));
}

#[test]
fn omitted_never_allow_defaults_to_uuid_and_an_empty_list_does_not() {
    expect(
        PATH,
        catalog("coupon-ids"),
        &format!("column:orders.coupon_ids: {NEVER}"),
    );
    let yaml = "\
schemaCatalogPath: schema.json
neverAllowElementTypes: []
allow:
  - object: column:orders.coupon_ids
    reason: this project stores uuid lists
";
    expect_none(yaml, catalog("coupon-ids"));
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
        catalog("accounts-display-name"),
        "stale postgres-array-columns allow entry: column:accounts.missing",
    );
}
