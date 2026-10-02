use super::support::{expect, expect_none, fixture, messages, NAMES, PATH, REVIEW_OPTIONS};

const LITERAL: &str = "text column holds a fixed set of values ('draft', 'sent', 'paid') enforced by CHECK; use an enum type or a foreign key to a lookup table";

#[test]
fn validated_check_sets_intersect_and_peer_index_excludes_non_candidates() {
    let messages = messages(REVIEW_OPTIONS, fixture("review-followups/schema.json"));
    let status = messages
        .iter()
        .find(|message| message.contains("column:accounts.status:"))
        .unwrap();
    assert!(status.contains("fixed set of values ('sent')"), "{status}");
    assert!(
        status.contains("accounts.peer_status has the same values"),
        "{status}"
    );
    assert!(!status.contains("foreign_status"), "{status}");
    assert!(!status.contains("generated_status"), "{status}");
    assert!(
        messages.iter().any(|message| {
            message.contains("column:accounts.bounded_state:")
                && message.contains("fixed set of values ('open', 'closed')")
        }),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .all(|message| !message.contains("column:accounts.malformed_state:")),
        "{messages:#?}"
    );
    assert!(
        messages.iter().any(|message| {
            message.contains("column:accounts.quoted_kind:") && message.contains("'O''Reilly'")
        }),
        "{messages:#?}"
    );
}

#[test]
fn pinned_text_columns_share_a_type_and_keep_first_seen_order() {
    let messages = messages(PATH, fixture("unit/scenarios/shared-values/schema.json"));
    assert!(
        messages.iter().any(|message| {
            message.contains("column:invoices.status: ")
                && message.contains(LITERAL)
                && message.contains("bills.state and quotes.status have the same values")
        }),
        "{messages:#?}"
    );
    assert!(
        messages.iter().any(|message| {
            message.contains("('paid', 'draft', 'sent')")
                && message.contains("invoices.status and quotes.status")
        }),
        "{messages:#?}"
    );
    assert_eq!(messages.len(), 3, "{messages:#?}");
}

#[test]
fn nullable_single_value_or_branches_and_one_finding() {
    expect(
        PATH,
        fixture("unit/scenarios/orders-nullable/schema.json"),
        "column:orders.kind: text column holds a fixed set of values ('a', 'b') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect(
        PATH,
        fixture("unit/scenarios/payments-single/schema.json"),
        "column:payments.currency: text column holds a fixed set of values ('USD') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect(
        PATH,
        fixture("unit/scenarios/jobs-scope/schema.json"),
        "column:jobs.scope: text column holds a fixed set of values ('global', 'tenant') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect(
        PATH,
        fixture("unit/scenarios/shipments/schema.json"),
        "column:shipments.state: text column holds a fixed set of values ('new', 'sent', 'lost') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
}

#[test]
fn composite_foreign_keys_stay_candidates_and_names_fill_the_gap() {
    let messages = messages(
        NAMES,
        fixture("unit/scenarios/composite-foreign-keys/schema.json"),
    );
    assert!(
        messages.iter().any(|message| message.contains(
            "column:order_links.link_kind: text column holds a fixed set of values ('order')"
        )),
        "{messages:#?}"
    );
    assert!(messages.iter().any(|message| message.contains(
        "column:accounts.plan_type: text column name matches (^|_)(status|state|kind|type|source|category|mode|channel)$"
    )), "{messages:#?}");
    assert_eq!(messages.len(), 2, "{messages:#?}");
}

#[test]
fn valid_columns_are_silent() {
    let yaml = "schemaCatalogPath: schema.json\nnamePatterns: ['(^|_)(status|state|kind|type|source|category|mode|channel)$']\nallow:\n  - object: column:webhook_deliveries.event_type\n    reason: Values are defined by the webhook provider and change without a migration\n";
    expect_none(yaml, fixture("unit/scenarios/valid-columns/schema.json"));
}

#[test]
fn edges_cover_casts_quotes_names_generated_and_truncation() {
    expect(
        PATH,
        fixture("unit/scenarios/labels-varying/schema.json"),
        "column:labels.kind: character varying column holds a fixed set of values ('open') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect(
        PATH,
        fixture("unit/scenarios/labels-quoted/schema.json"),
        "column:labels.Kind: text column holds a fixed set of values ('a', 'b') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    let pinned_name = messages(NAMES, fixture("unit/scenarios/pinned-name/schema.json"));
    assert_eq!(pinned_name.len(), 1, "{pinned_name:#?}");
    assert!(!pinned_name[0].contains("name matches"), "{pinned_name:#?}");
    expect_none(PATH, fixture("unit/scenarios/null-kind/schema.json"));
    expect(
        NAMES,
        fixture("unit/scenarios/null-kind/schema.json"),
        "column:accounts.kind: text column name matches (^|_)(status|state|kind|type|source|category|mode|channel)$, which marks a fixed set of values; use an enum type or a foreign key to a lookup table, or add an allow entry with a reason",
    );
    let generated = fixture("unit/scenarios/generated/schema.json");
    expect(
        PATH,
        generated.clone(),
        "column:entities.status: text column holds a fixed set of values ('on') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect_none(
        "schemaCatalogPath: schema.json\nskipGeneratedColumns: true\n",
        generated,
    );
    expect_none(
        "schemaCatalogPath: schema.json\nnamePatterns: []\n",
        fixture("unit/scenarios/plan-type-no-name/schema.json"),
    );
    expect_none(
        "schemaCatalogPath: schema.json\nignoreTablePatterns: ['^archive_']\n",
        fixture("unit/scenarios/archive-ignored/schema.json"),
    );
    expect(
        PATH,
        fixture("unit/scenarios/partitioned/schema.json"),
        "column:parts.status: TEXT column holds a fixed set of values ('open') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
}

#[test]
fn one_peer_and_three_peers_use_the_matching_join() {
    let pair = messages(PATH, fixture("unit/scenarios/peer-pair/schema.json"));
    assert!(
        pair.iter()
            .any(|message| message.contains("bills.status has the same values")),
        "{pair:#?}"
    );
    let messages = messages(PATH, fixture("unit/scenarios/peer-multiple/schema.json"));
    assert!(
        messages.iter().any(|message| {
            message.contains("b.status, c.status and d.status have the same values")
        }),
        "{messages:#?}"
    );
}

#[test]
fn long_value_lists_and_peer_lists_truncate() {
    let message = messages(PATH, fixture("unit/scenarios/long-values/schema.json"));
    assert_eq!(message.len(), 1);
    assert!(
        message[0]
            .contains("'v0', 'v1', 'v2', 'v3', 'v4', 'v5', 'v6', 'v7', 'v8', 'v9', … (2 more)"),
        "{}",
        message[0]
    );
    let messages = messages(PATH, fixture("unit/scenarios/long-peers/schema.json"));
    assert!(
        messages.iter().any(|message| {
            message
                .contains("t1.status, t2.status, t3.status, t4.status, t5.status and 1 more have")
        }),
        "{messages:#?}"
    );
}

#[test]
fn allow_suppresses_a_finding_and_reports_stale_entries() {
    let yaml = "schemaCatalogPath: schema.json\nallow:\n  - object: column:payments.currency\n    reason: ISO code pinned by the payments provider\n  - object: column:payments.missing\n    reason: leftover\n";
    let messages = messages(yaml, fixture("unit/scenarios/allow-stale/schema.json"));
    assert_eq!(
        messages,
        vec![
            "schema.json: stale postgres-finite-text-columns allow entry: column:payments.missing"
                .to_string()
        ]
    );
}

#[test]
fn two_check_sets_intersect_in_name_order() {
    expect(
        PATH,
        fixture("unit/scenarios/two-checks/schema.json"),
        "column:invoices.status: text column holds a fixed set of values ('sent', 'paid') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
}
