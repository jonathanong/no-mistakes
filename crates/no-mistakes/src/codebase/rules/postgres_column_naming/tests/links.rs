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
    let wrong = super::support::fixture_body("scenarios/links-00.json");
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
    let body = super::support::fixture_body("scenarios/links-01.json");
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
    let line = super::support::fixture_body("scenarios/links-02.json");
    expect(
        RESERVED,
        line.clone(),
        &at(
            "line_grants",
            "user_id",
            &format!("{RESERVED_USERS}; this column is only part of a composite foreign key to account_members"),
        ),
    );
    let leading = super::support::fixture_body("scenarios/composite-leading-targets.json");
    expect_none(&follow(true), leading);
    let dead = super::support::fixture_body("scenarios/links-04.json");
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
    let chain = super::support::fixture_body("scenarios/links-05.json");
    expect_none(&follow(true), chain);
    let renamed = super::support::fixture_body("scenarios/links-06.json");
    expect_none(&follow(true), renamed);
    let either = super::support::fixture_body("scenarios/links-07.json");
    expect_none(&follow(true), either);
    let cycle = super::support::fixture_body("scenarios/links-08.json");
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
        super::support::fixture_body("scenarios/links-09.json"),
    );
    expect_none(REQUIRE, column("ids", "thing_id", "uuid[]"));
    expect_none(
        REQUIRE,
        super::support::fixture_body("scenarios/links-10.json"),
    );
    expect_none(
        REQUIRE,
        super::support::fixture_body("scenarios/links-11.json"),
    );
    expect_none(
        REQUIRE,
        super::support::fixture_body("scenarios/links-12.json"),
    );
    expect_none(
        REQUIRE,
        super::support::fixture_body("scenarios/links-13.json"),
    );
    let cursor = r#"
schemaCatalogPath: schema.json
foreignKeys:
  requireForeignKey:
    types: [uuid]
    namePattern: '_id$'
    exempt:
      - namePattern: '^cursor_'
        reason: cursors
"#;
    let stale =
        "schema.json: stale postgres-column-naming requireForeignKey exempt entry: ^cursor_";
    expect(cursor, column("sync", "cursor_name", "text"), stale);
    expect(
        cursor,
        super::support::fixture_body("scenarios/links-14.json"),
        stale,
    );
    expect(
        "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: [uuid]\n    namePattern: _id$\n    exempt:\n      - namePattern: '^never$'\n        reason: unused\n",
        super::support::fixture_body("scenarios/links-15.json"),
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
        super::support::fixture_body("scenarios/links-16.json"),
        "schema.json: stale postgres-column-naming requireForeignKey exempt entry: ^only_on_link",
    );
}
