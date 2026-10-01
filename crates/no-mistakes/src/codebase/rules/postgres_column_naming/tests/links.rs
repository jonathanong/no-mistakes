use super::support::{at, column, expect, expect_none, messages, REQUIRE, RESERVED};

const RESERVED_USERS: &str =
    "_user_id is reserved for foreign keys to users or deleted_user_identities";

fn follow(on: bool) -> String {
    RESERVED.replace(
        "followCompositeForeignKeys: false",
        &format!("followCompositeForeignKeys: {on}"),
    )
}

#[test]
fn reserved_suffixes_name_the_reason_the_column_fails() {
    expect(
        RESERVED,
        column("sessions", "reviewer_user_id", "uuid"),
        &at(
            "sessions",
            "reviewer_user_id",
            &format!("{RESERVED_USERS}; this uuid column has no foreign key"),
        ),
    );
    expect(
        RESERVED,
        column("sessions", "user_id", "uuid"),
        &at(
            "sessions",
            "user_id",
            &format!("{RESERVED_USERS}; this uuid column has no foreign key"),
        ),
    );
    expect_none(
        RESERVED,
        column("linked_accounts", "github_user_id", "text"),
    );
    expect(
        RESERVED,
        column("bookmarks", "page_url", "text"),
        &at(
            "bookmarks",
            "page_url",
            "_url is reserved for foreign keys to links; this text column has no foreign key; store the URL in links and reference it with a <name>_link_id column",
        ),
    );
    expect(
        RESERVED,
        column("links", "url", "text"),
        &at(
            "links",
            "url",
            "_url is reserved for foreign keys to links; this text column has no foreign key; store the URL in links and reference it with a <name>_link_id column",
        ),
    );
    expect_none(RESERVED, column("bookmarks", "page_url_id", "uuid"));
    let any_type = "schemaCatalogPath: schema.json\nforeignKeys:\n  reservedSuffixes:\n    - suffix: _user_id\n      tables: [users]\n";
    expect(
        any_type,
        column("sessions", "reviewer_user_id", "text"),
        &at(
            "sessions",
            "reviewer_user_id",
            "_user_id is reserved for foreign keys to users; this text column has no foreign key",
        ),
    );
    let wrong = serde_json::json!({
        "tables": {
            "tasks": {
                "columns": { "assignee_user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": { "columns": ["assignee_user_id"], "referencedTable": "teams", "referencedColumns": ["id"] } }
            }
        }
    });
    expect(
        RESERVED,
        wrong,
        &at(
            "tasks",
            "assignee_user_id",
            &format!("{RESERVED_USERS}; this column references teams"),
        ),
    );
}

#[test]
fn assignee_user_id_gets_reserved_and_target_name_findings() {
    let yaml = r#"
schemaCatalogPath: schema.json
foreignKeys:
  targetMatch: last-word
  reservedSuffixes:
    - suffix: '_user_id'
      types: [uuid]
      tables: [users, deleted_user_identities]
"#;
    let body = serde_json::json!({
        "tables": {
            "tasks": {
                "columns": { "assignee_user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": { "columns": ["assignee_user_id"], "referencedTable": "teams", "referencedColumns": ["id"] } }
            }
        }
    });
    assert_eq!(
        messages(yaml, body),
        vec![
            at(
                "tasks",
                "assignee_user_id",
                &format!("{RESERVED_USERS}; this column references teams")
            ),
            at(
                "tasks",
                "assignee_user_id",
                "foreign key to teams must end in team_id (for example assignee_user_team_id)"
            ),
        ]
    );
}

#[test]
fn composite_foreign_keys_follow_only_when_asked() {
    let line = serde_json::json!({
        "tables": {
            "line_grants": {
                "columns": {
                    "account_id": { "dataType": "uuid" },
                    "user_id": { "dataType": "uuid" }
                },
                "foreignKeys": {
                    "fk": {
                        "columns": ["account_id", "user_id"],
                        "referencedTable": "account_members",
                        "referencedColumns": ["account_id", "user_id"]
                    }
                }
            }
        }
    });
    expect(
        RESERVED,
        line.clone(),
        &at(
            "line_grants",
            "user_id",
            &format!("{RESERVED_USERS}; this column is only part of a composite foreign key to account_members"),
        ),
    );
    let leading = serde_json::json!({
        "tables": {
            "line_grants": line["tables"]["line_grants"].clone(),
            "account_members": {
                "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "user_fk": { "columns": ["user_id"], "referencedTable": "users", "referencedColumns": ["id"] } }
            }
        }
    });
    expect_none(&follow(true), leading);
    let dead = serde_json::json!({
        "tables": {
            "seat_grants": {
                "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": {
                    "fk": {
                        "columns": ["account_id", "user_id"],
                        "referencedTable": "account_seats",
                        "referencedColumns": ["account_id", "seat_id"]
                    }
                }
            },
            "account_seats": { "columns": { "account_id": { "dataType": "uuid" }, "seat_id": { "dataType": "uuid" } } }
        }
    });
    expect(
        &follow(true),
        dead,
        &at(
            "seat_grants",
            "user_id",
            &format!("{RESERVED_USERS}; this column is part of a composite foreign key to account_seats, which does not lead to users or deleted_user_identities"),
        ),
    );
}

#[test]
fn follow_walks_two_hops_renames_and_stops_on_cycles() {
    let chain = serde_json::json!({
        "tables": {
            "grant_uses": {
                "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": { "columns": ["account_id", "user_id"], "referencedTable": "line_grants", "referencedColumns": ["account_id", "user_id"] } }
            },
            "line_grants": {
                "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": { "columns": ["account_id", "user_id"], "referencedTable": "account_members", "referencedColumns": ["account_id", "user_id"] } }
            },
            "account_members": {
                "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "user_fk": { "columns": ["user_id"], "referencedTable": "users", "referencedColumns": ["id"] } }
            }
        }
    });
    expect_none(&follow(true), chain);
    let renamed = serde_json::json!({
        "tables": {
            "grants": {
                "columns": { "account_id": { "dataType": "uuid" }, "member_user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": { "columns": ["account_id", "member_user_id"], "referencedTable": "account_members", "referencedColumns": ["account_id", "user_id"] } }
            },
            "account_members": {
                "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "user_fk": { "columns": ["user_id"], "referencedTable": "users", "referencedColumns": ["id"] } }
            }
        }
    });
    expect_none(&follow(true), renamed);
    let either = serde_json::json!({
        "tables": {
            "grants": {
                "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": {
                    "a": { "columns": ["account_id", "user_id"], "referencedTable": "account_seats", "referencedColumns": ["account_id", "seat_id"] },
                    "b": { "columns": ["team_id", "user_id"], "referencedTable": "account_members", "referencedColumns": ["account_id", "user_id"] }
                }
            },
            "account_seats": { "columns": { "seat_id": { "dataType": "uuid" } } },
            "account_members": {
                "columns": { "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "user_fk": { "columns": ["user_id"], "referencedTable": "users", "referencedColumns": ["id"] } }
            }
        }
    });
    expect_none(&follow(true), either);
    let cycle = serde_json::json!({
        "tables": {
            "a": {
                "columns": { "k": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": { "columns": ["k", "user_id"], "referencedTable": "b", "referencedColumns": ["k", "user_id"] } }
            },
            "b": {
                "columns": { "k": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": { "columns": ["k", "user_id"], "referencedTable": "a", "referencedColumns": ["k", "user_id"] } }
            }
        }
    });
    let text = |table: &str| {
        format!("{RESERVED_USERS}; this column is part of a composite foreign key to {table}, which does not lead to users or deleted_user_identities")
    };
    assert_eq!(
        messages(&follow(true), cycle),
        vec![
            at("a", "user_id", &text("b")),
            at("b", "user_id", &text("a"))
        ]
    );
}

#[test]
fn require_foreign_key_counts_every_position_except_generated_and_reserved() {
    expect(
        REQUIRE,
        column("audit_rows", "order_id", "uuid"),
        &at(
            "audit_rows",
            "order_id",
            "uuid column name matches _id$ but the column is not part of any foreign key; add a foreign key, or an exempt pattern or allow entry with a reason",
        ),
    );
    expect_none(
        "schemaCatalogPath: schema.json\n",
        column("audit_rows", "order_id", "uuid"),
    );
    let exempt = r#"
schemaCatalogPath: schema.json
foreignKeys:
  requireForeignKey:
    types: [uuid]
    namePattern: '_id$'
    exempt:
      - namePattern: '^cursor_'
        reason: 'Keyset cursors may point at rows that are already deleted'
      - namePattern: '(^|_)(device|session)_id$'
        reason: 'Device and session ids are issued in signed tokens, not stored in a table'
"#;
    expect_none(
        exempt,
        serde_json::json!({
            "tables": {
                "sync_cursors": { "columns": { "cursor_id": { "dataType": "uuid" }, "cursor_order_id": { "dataType": "uuid" } } },
                "login_events": { "columns": { "session_id": { "dataType": "uuid" } } }
            }
        }),
    );
    expect_none(REQUIRE, column("ids", "thing_id", "uuid[]"));
    expect_none(
        REQUIRE,
        serde_json::json!({
            "tables": { "entities": { "columns": { "entity_id": { "dataType": "uuid", "generated": "stored" } } } }
        }),
    );
    expect_none(
        REQUIRE,
        serde_json::json!({
            "tables": {
                "projects": {
                    "columns": { "owner_id": { "dataType": "uuid" } },
                    "foreignKeys": { "fk": { "columns": ["owner_id"], "referencedTable": "users", "referencedColumns": ["id"] } }
                }
            }
        }),
    );
    expect_none(
        REQUIRE,
        serde_json::json!({
            "tables": {
                "line_grants": {
                    "columns": { "account_id": { "dataType": "uuid" }, "user_id": { "dataType": "uuid" } },
                    "foreignKeys": { "fk": { "columns": ["account_id", "user_id"], "referencedTable": "account_members", "referencedColumns": ["account_id", "user_id"] } }
                }
            }
        }),
    );
    expect_none(
        REQUIRE,
        serde_json::json!({
            "tables": {
                "categories": {
                    "columns": { "parent_id": { "dataType": "uuid" } },
                    "foreignKeys": { "fk": { "columns": ["parent_id"], "referencedTable": "categories", "referencedColumns": ["id"] } }
                }
            }
        }),
    );
    expect(
        "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: [uuid]\n    namePattern: _id$\n    exempt:\n      - namePattern: '^never$'\n        reason: unused\n",
        serde_json::json!({ "tables": {} }),
        "schema.json: stale postgres-column-naming requireForeignKey exempt entry: ^never$",
    );
    let both = r#"
schemaCatalogPath: schema.json
foreignKeys:
  reservedSuffixes:
    - suffix: '_user_id'
      types: [uuid]
      tables: [users]
  requireForeignKey:
    types: [uuid]
    namePattern: _id$
"#;
    let messages = messages(both, column("sessions", "reviewer_user_id", "uuid"));
    assert_eq!(messages.len(), 1, "{messages:#?}");
    assert!(
        messages[0].contains("this uuid column has no foreign key"),
        "{messages:#?}"
    );
}

#[test]
fn ignored_tables_produce_no_findings_and_do_not_use_exempt_patterns() {
    let yaml = r#"
schemaCatalogPath: schema.json
ignoreTablePatterns: ['^link__']
foreignKeys:
  reservedSuffixes:
    - suffix: '_user_id'
      types: [uuid]
      tables: [users]
  requireForeignKey:
    types: [uuid]
    namePattern: _id$
    exempt:
      - namePattern: '^only_on_link'
        reason: generated table
"#;
    expect(
        yaml,
        serde_json::json!({
            "tables": {
                "link__accounts": {
                    "columns": {
                        "only_on_link": { "dataType": "uuid" },
                        "reviewer_user_id": { "dataType": "uuid" }
                    }
                }
            }
        }),
        "schema.json: stale postgres-column-naming requireForeignKey exempt entry: ^only_on_link",
    );
}
