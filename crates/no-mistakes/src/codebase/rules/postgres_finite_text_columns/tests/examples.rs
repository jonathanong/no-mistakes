use super::support::{checked, expect, expect_none, messages, NAMES, PATH};

const LITERAL: &str = "text column holds a fixed set of values ('draft', 'sent', 'paid') enforced by CHECK; use an enum type or a foreign key to a lookup table";

#[test]
fn pinned_text_columns_share_a_type_and_keep_first_seen_order() {
    let body = serde_json::json!({
        "tables": {
            "invoices": {
                "columns": { "status": { "dataType": "text" } },
                "checkConstraints": {
                    "ck": { "definition": "CHECK ((status = ANY (ARRAY['draft'::text, 'sent'::text, 'paid'::text])))" }
                }
            },
            "bills": {
                "columns": { "state": { "dataType": "text" } },
                "checkConstraints": {
                    "ck": { "definition": "CHECK ((state = ANY (ARRAY['paid'::text, 'draft'::text, 'sent'::text])))" }
                }
            },
            "quotes": {
                "columns": { "status": { "dataType": "text" } },
                "checkConstraints": {
                    "ck": { "definition": "CHECK ((status IN ('sent', 'paid', 'draft')))" }
                }
            }
        }
    });
    let messages = messages(PATH, body);
    assert!(
        messages
            .iter()
            .any(|message| message.contains("column:invoices.status: ")
                && message.contains(LITERAL)
                && message.contains("bills.state and quotes.status have the same values")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("('paid', 'draft', 'sent')")
                && message.contains("invoices.status and quotes.status")),
        "{messages:#?}"
    );
    assert_eq!(messages.len(), 3, "{messages:#?}");
}

#[test]
fn nullable_single_value_or_branches_and_one_finding() {
    expect(
        PATH,
        checked("orders", "kind", "text", "CHECK (((kind IS NULL) OR (kind = ANY (ARRAY['a'::text, 'b'::text]))))"),
        "column:orders.kind: text column holds a fixed set of values ('a', 'b') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect(
        PATH,
        checked("payments", "currency", "text", "CHECK ((currency = 'USD'::text))"),
        "column:payments.currency: text column holds a fixed set of values ('USD') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect(
        PATH,
        checked(
            "jobs",
            "scope",
            "text",
            "CHECK ((((scope = 'global'::text) AND (tenant_id IS NULL)) OR ((scope = 'tenant'::text) AND (tenant_id IS NOT NULL))))",
        ),
        "column:jobs.scope: text column holds a fixed set of values ('global', 'tenant') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    let body = serde_json::json!({
        "tables": {
            "shipments": {
                "columns": { "state": { "dataType": "text" }, "lost_at": { "dataType": "timestamp with time zone" } },
                "checkConstraints": {
                    "ck_extra": { "definition": "CHECK (((state <> 'lost'::text) OR (lost_at IS NOT NULL)))" },
                    "ck_values": { "definition": "CHECK ((state = ANY (ARRAY['new'::text, 'sent'::text, 'lost'::text])))" }
                }
            }
        }
    });
    expect(
        PATH,
        body,
        "column:shipments.state: text column holds a fixed set of values ('new', 'sent', 'lost') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
}

#[test]
fn composite_foreign_keys_stay_candidates_and_names_fill_the_gap() {
    let body = serde_json::json!({
        "tables": {
            "order_links": {
                "columns": { "order_id": { "dataType": "uuid" }, "link_kind": { "dataType": "text" } },
                "foreignKeys": { "fk": { "columns": ["order_id", "link_kind"], "referencedTable": "orders", "referencedColumns": ["id", "kind"] } },
                "checkConstraints": { "ck": { "definition": "CHECK ((link_kind = 'order'::text))" } }
            },
            "accounts": { "columns": { "plan_type": { "dataType": "text" } } }
        }
    });
    let messages = messages(NAMES, body);
    assert!(
        messages.iter().any(|message| message.contains(
            "column:order_links.link_kind: text column holds a fixed set of values ('order')"
        )),
        "{messages:#?}"
    );
    assert!(messages.iter().any(|message| message.contains("column:accounts.plan_type: text column name matches (^|_)(status|state|kind|type|source|category|mode|channel)$")), "{messages:#?}");
    assert_eq!(messages.len(), 2, "{messages:#?}");
}

#[test]
fn valid_columns_are_silent() {
    let body = serde_json::json!({
        "tables": {
            "invoices": {
                "columns": {
                    "status": { "dataType": "invoice_statuses" },
                    "invoice_status_id": { "dataType": "uuid" }
                },
                "foreignKeys": { "fk": { "columns": ["invoice_status_id"], "referencedTable": "invoice_statuses", "referencedColumns": ["id"] } }
            },
            "accounts": {
                "columns": {
                    "plan_code": { "dataType": "text" },
                    "display_name": { "dataType": "text" },
                    "score": { "dataType": "integer" }
                },
                "foreignKeys": { "fk": { "columns": ["plan_code"], "referencedTable": "plans", "referencedColumns": ["code"] } },
                "checkConstraints": {
                    "ck_code": { "definition": "CHECK ((plan_code = 'free'::text))" },
                    "ck_score": { "definition": "CHECK ((score = ANY (ARRAY['-1'::integer, 0, 1])))" },
                    "ck_status": { "definition": "CHECK ((status <> ALL (ARRAY['deleted'::text])))" },
                    "ck_mixed": { "definition": "CHECK ((status = ANY (ARRAY[other_status, 'x'::text])))" },
                    "ck_or": { "definition": "CHECK (((kind <> 'refund'::text) OR (reason_code = 'fraud'::text)))" },
                    "ck_len": { "definition": "CHECK ((char_length(label) <= 100))" }
                }
            },
            "webhook_deliveries": { "columns": { "event_type": { "dataType": "text" } } }
        }
    });
    let yaml = "\
schemaCatalogPath: schema.json
namePatterns: ['(^|_)(status|state|kind|type|source|category|mode|channel)$']
allow:
  - object: column:webhook_deliveries.event_type
    reason: Values are defined by the webhook provider and change without a migration
";
    expect_none(yaml, body);
}

#[test]
fn edges_cover_casts_quotes_names_generated_and_truncation() {
    expect(
        PATH,
        checked(
            "labels",
            "kind",
            "character varying",
            "CHECK ((kind::character varying = 'open'::character varying))",
        ),
        "column:labels.kind: character varying column holds a fixed set of values ('open') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    expect(
        PATH,
        checked("labels", "Kind", "text", r#"CHECK (("Kind" IN ('a', 'b')))"#),
        "column:labels.Kind: text column holds a fixed set of values ('a', 'b') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
    let pinned_name = checked(
        "accounts",
        "status",
        "text",
        "CHECK ((status = ANY (ARRAY['draft'::text])))",
    );
    let messages = messages(NAMES, pinned_name);
    assert_eq!(messages.len(), 1, "{messages:#?}");
    assert!(!messages[0].contains("name matches"), "{messages:#?}");
    expect_none(
        PATH,
        checked("accounts", "kind", "text", "CHECK ((kind IS NULL))"),
    );
    expect(
        NAMES,
        checked("accounts", "kind", "text", "CHECK ((kind IS NULL))"),
        "column:accounts.kind: text column name matches (^|_)(status|state|kind|type|source|category|mode|channel)$, which marks a fixed set of values; use an enum type or a foreign key to a lookup table, or add an allow entry with a reason",
    );
    let generated = serde_json::json!({
        "tables": {
            "entities": {
                "columns": { "status": { "dataType": "text", "generated": "stored" } },
                "checkConstraints": { "ck": { "definition": "CHECK ((status = 'on'::text))" } }
            }
        }
    });
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
        serde_json::json!({"tables": {"accounts": {"columns": {"plan_type": {"dataType": "text"}}}}}),
    );
    expect_none(
        "schemaCatalogPath: schema.json\nignoreTablePatterns: ['^archive_']\n",
        checked(
            "archive_invoices",
            "status",
            "text",
            "CHECK ((status = 'old'::text))",
        ),
    );
    expect(
        PATH,
        serde_json::json!({"tables": {"parts": {"relationKind": "partitioned table", "columns": {"status": {"dataType": "TEXT"}}, "checkConstraints": {"ck": {"definition": "CHECK ((status = 'open'::text))"}}}}}),
        "column:parts.status: TEXT column holds a fixed set of values ('open') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
}

#[test]
fn one_peer_and_three_peers_use_the_matching_join() {
    let definition = "CHECK ((status = ANY (ARRAY['a'::text, 'b'::text])))";
    let pair = messages(
        PATH,
        serde_json::json!({
            "tables": {
                "bills": {
                    "columns": { "status": { "dataType": "text" } },
                    "checkConstraints": { "ck": { "definition": definition } }
                },
                "invoices": {
                    "columns": { "status": { "dataType": "text" } },
                    "checkConstraints": { "ck": { "definition": definition } }
                }
            }
        }),
    );
    assert!(
        pair.iter()
            .any(|message| message.contains("bills.status has the same values")),
        "{pair:#?}"
    );
    let mut tables = serde_json::Map::new();
    for name in ["a", "b", "c", "d"] {
        tables.insert(
            name.to_string(),
            serde_json::json!({
                "columns": { "status": { "dataType": "text" } },
                "checkConstraints": { "ck": { "definition": definition } }
            }),
        );
    }
    let messages = messages(PATH, serde_json::json!({ "tables": tables }));
    assert!(
        messages.iter().any(|message| message
            .contains("b.status, c.status and d.status have the same values")),
        "{messages:#?}"
    );
}

#[test]
fn long_value_lists_and_peer_lists_truncate() {
    let values = (0..12).map(|index| format!("v{index}")).collect::<Vec<_>>();
    let list = values
        .iter()
        .map(|value| format!("'{value}'::text"))
        .collect::<Vec<_>>()
        .join(", ");
    let definition = format!("CHECK ((status = ANY (ARRAY[{list}])))");
    let message = messages(PATH, checked("invoices", "status", "text", &definition));
    assert_eq!(message.len(), 1);
    assert!(
        message[0]
            .contains("'v0', 'v1', 'v2', 'v3', 'v4', 'v5', 'v6', 'v7', 'v8', 'v9', … (2 more)"),
        "{}",
        message[0]
    );
    let mut tables = serde_json::Map::new();
    for index in 0..7 {
        let name = format!("t{index}");
        tables.insert(name, serde_json::json!({
            "columns": { "status": { "dataType": "text" } },
            "checkConstraints": { "ck": { "definition": "CHECK ((status = ANY (ARRAY['a'::text, 'b'::text])))" } }
        }));
    }
    let messages = messages(PATH, serde_json::json!({ "tables": tables }));
    assert!(
        messages.iter().any(|message| message
            .contains("t1.status, t2.status, t3.status, t4.status, t5.status and 1 more have")),
        "{messages:#?}"
    );
}

#[test]
fn allow_suppresses_a_finding_and_reports_stale_entries() {
    let yaml = "\
schemaCatalogPath: schema.json
allow:
  - object: column:payments.currency
    reason: ISO code pinned by the payments provider
  - object: column:payments.missing
    reason: leftover
";
    let messages = messages(
        yaml,
        checked(
            "payments",
            "currency",
            "text",
            "CHECK ((currency = 'USD'::text))",
        ),
    );
    assert_eq!(
        messages,
        vec![
            "schema.json: stale postgres-finite-text-columns allow entry: column:payments.missing"
                .to_string()
        ]
    );
}

#[test]
fn two_checks_union_in_name_order() {
    let body = serde_json::json!({
        "tables": {
            "invoices": {
                "columns": { "status": { "dataType": "text" } },
                "checkConstraints": {
                    "ck_narrow": { "definition": "CHECK (((status = 'sent'::text) OR (status = 'paid'::text)))" },
                    "ck_domain": { "definition": "CHECK ((status = ANY (ARRAY['draft'::text, 'sent'::text, 'paid'::text])))" }
                }
            }
        }
    });
    expect(
        PATH,
        body,
        "column:invoices.status: text column holds a fixed set of values ('draft', 'sent', 'paid') enforced by CHECK; use an enum type or a foreign key to a lookup table",
    );
}
