use super::support::{at, expect, expect_none, sole_fk, LAST_WORD, SUFFIX};

#[test]
fn suffix_rule_skips_behaviour_three_and_suggests_the_first_suffix() {
    expect(
        SUFFIX,
        sole_fk("projects", "owner_id", "uuid", "users", "id"),
        &at(
            "projects",
            "owner_id",
            "foreign key to users must end in _user_id or _by_id (for example owner_user_id)",
        ),
    );
    for (table, column, referenced) in [
        ("projects", "owner_user_id", "users"),
        ("orders", "created_by_id", "users"),
        ("api_keys", "revoked_by_id", "deleted_user_identities"),
    ] {
        expect_none(SUFFIX, sole_fk(table, column, "uuid", referenced, "id"));
    }
}

#[test]
fn exact_user_id_does_not_satisfy_the_suffix_rule() {
    expect(
        SUFFIX,
        sole_fk("account_members", "user_id", "uuid", "users", "id"),
        &at(
            "account_members",
            "user_id",
            "foreign key to users must end in _user_id or _by_id (for example user_user_id)",
        ),
    );
}

#[test]
fn target_match_off_last_word_and_full_name() {
    let item = sole_fk("bookmarks", "item_id", "uuid", "articles", "id");
    let batch = sole_fk("import_rows", "batch_id", "uuid", "import_batches", "id");
    expect_none(
        "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: off\n",
        item.clone(),
    );
    expect_none(
        "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: off\n",
        batch.clone(),
    );
    expect(
        LAST_WORD,
        item.clone(),
        &at(
            "bookmarks",
            "item_id",
            "foreign key to articles must end in article_id (for example item_article_id)",
        ),
    );
    expect_none(LAST_WORD, batch.clone());
    let full = "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: full-name\n";
    expect(
        full,
        batch,
        &at(
            "import_rows",
            "batch_id",
            "foreign key to import_batches must end in import_batch_id (for example import_batch_id)",
        ),
    );
    expect_none(
        full,
        sole_fk(
            "import_rows",
            "import_batch_id",
            "uuid",
            "import_batches",
            "id",
        ),
    );
    expect(
        full,
        item,
        &at(
            "bookmarks",
            "item_id",
            "foreign key to articles must end in article_id (for example item_article_id)",
        ),
    );
}

#[test]
fn suggestions_and_singular_targets_are_verbatim() {
    expect(
        LAST_WORD,
        sole_fk("orders", "currency", "text", "currencies", "code"),
        &at(
            "orders",
            "currency",
            "foreign key to currencies must end in currency_code (for example currency_code)",
        ),
    );
    expect_none(
        LAST_WORD,
        sole_fk("orders", "currency_code", "text", "currencies", "code"),
    );
    expect(
        LAST_WORD,
        sole_fk("orders", "order_currency", "text", "currencies", "code"),
        &at(
            "orders",
            "order_currency",
            "foreign key to currencies must end in currency_code (for example order_currency_code)",
        ),
    );
    expect(
        LAST_WORD,
        sole_fk("snapshot_notes", "snapshot_id", "uuid", "archived_order_snapshots", "id"),
        &at(
            "snapshot_notes",
            "snapshot_id",
            "foreign key to archived_order_snapshots must end in order_id (for example snapshot_order_id)",
        ),
    );
    expect_none(
        LAST_WORD,
        sole_fk(
            "snapshot_notes",
            "order_id",
            "uuid",
            "archived_order_snapshots",
            "id",
        ),
    );
    expect_none(
        LAST_WORD,
        sole_fk("bookmarks", "article_id", "uuid", "articles", "id"),
    );
    expect_none(
        LAST_WORD,
        sole_fk("order_line_items", "order_id", "uuid", "orders", "id"),
    );
    expect(
        LAST_WORD,
        sole_fk("friends", "friend_id", "uuid", "people", "id"),
        &at(
            "friends",
            "friend_id",
            "foreign key to people must end in person_id (for example friend_person_id)",
        ),
    );
    expect(
        LAST_WORD,
        sole_fk("rows", "row_id", "uuid", "statuses", "id"),
        &at(
            "rows",
            "row_id",
            "foreign key to statuses must end in status_id (for example row_status_id)",
        ),
    );
    expect(
        LAST_WORD,
        sole_fk("members", "member_id", "uuid", "staff", "id"),
        &at(
            "members",
            "member_id",
            "foreign key to staff must end in staff_id (for example member_staff_id)",
        ),
    );
}

#[test]
fn descriptive_keys_id_columns_and_self_references() {
    expect(
        LAST_WORD,
        sole_fk(
            "linked_accounts",
            "github_user_id",
            "text",
            "github_accounts",
            "github_user_id",
        ),
        &at(
            "linked_accounts",
            "github_user_id",
            "foreign key to github_accounts must end in account_github_user_id (for example github_user_id_account_github_user_id)",
        ),
    );
    expect_none(
        LAST_WORD,
        sole_fk(
            "linked_accounts",
            "github_account_id",
            "text",
            "github_accounts",
            "github_account_id",
        ),
    );
    expect_none(
        "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: full-name\n",
        sole_fk(
            "rows",
            "import_batch_id",
            "text",
            "import_batches",
            "import_batch_id",
        ),
    );
    expect_none(
        "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: off\n",
        sole_fk(
            "linked_accounts",
            "github_user_id",
            "text",
            "github_accounts",
            "github_user_id",
        ),
    );
    expect(
        LAST_WORD,
        sole_fk("payments", "external_id", "text", "orders", "external_id"),
        &at(
            "payments",
            "external_id",
            "foreign key to orders must end in order_external_id (for example external_id_order_external_id)",
        ),
    );
    expect_none(
        LAST_WORD,
        sole_fk(
            "linked_accounts",
            "account_github_user_id",
            "text",
            "github_accounts",
            "github_user_id",
        ),
    );
    expect_none(
        LAST_WORD,
        sole_fk("premium_accounts", "id", "uuid", "accounts", "id"),
    );
    expect_none(
        LAST_WORD,
        sole_fk("categories", "parent_id", "uuid", "categories", "id"),
    );
    expect(
        "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: last-word\n  checkSelfReferences: true\n",
        sole_fk("categories", "parent_id", "uuid", "categories", "id"),
        &at(
            "categories",
            "parent_id",
            "foreign key to categories must end in category_id (for example parent_category_id)",
        ),
    );
    expect_none(
        LAST_WORD,
        super::support::fixture_body("scenarios/foreign-00.json"),
    );
}

#[test]
fn schema_qualified_targets_natural_keys_and_composite_targets_are_resolved() {
    let yaml = r#"
schemaCatalogPath: schema.json
foreignKeys:
  targetMatch: last-word
  reservedSuffixes:
    - suffix: '_user_id'
      types: [uuid]
      tables: [users]
  followCompositeForeignKeys: true
"#;
    assert_eq!(
        super::support::fixture_messages(yaml, "qualified-composite-and-natural-keys.json"),
        vec![at(
            "payments",
            "external_id",
            "foreign key to orders must end in order_external_id (for example external_id_order_external_id)"
        )]
    );
}

#[test]
fn last_word_mode_accepts_a_repeated_natural_key() {
    assert!(super::support::fixture_messages(LAST_WORD, "repeated-natural-key.json").is_empty());
}

#[test]
fn target_name_substitution_uses_the_first_pattern() {
    let yaml = r#"
schemaCatalogPath: schema.json
foreignKeys:
  targetMatch: last-word
  targetNames:
    - tablePattern: '^arch'
      name: early
    - tablePattern: '^archived_(.+)_snapshots$'
      name: '$1'
"#;
    expect(
        yaml,
        sole_fk("notes", "snapshot_id", "uuid", "archived_order_snapshots", "id"),
        &at(
            "notes",
            "snapshot_id",
            "foreign key to archived_order_snapshots must end in early_id (for example snapshot_early_id)",
        ),
    );
    let missing = r#"
schemaCatalogPath: schema.json
foreignKeys:
  targetMatch: last-word
  targetNames:
    - tablePattern: '^(archived_)?([a-z]+)$'
      name: '$1$2'
    - tablePattern: '^t_(.+)$'
      name: 'item$0'
"#;
    expect(
        missing,
        sole_fk("notes", "note_id", "uuid", "orders", "id"),
        &at(
            "notes",
            "note_id",
            "foreign key to orders must end in orders_id (for example note_orders_id)",
        ),
    );
    expect(
        missing,
        sole_fk("notes", "note_id", "uuid", "t_order", "id"),
        &at(
            "notes",
            "note_id",
            "foreign key to t_order must end in item$0_id (for example note_item$0_id)",
        ),
    );
    expect(
        LAST_WORD,
        sole_fk("odd", "_id", "uuid", "articles", "id"),
        &at(
            "odd",
            "_id",
            "foreign key to articles must end in article_id (for example article_id)",
        ),
    );
}

#[test]
fn a_quoted_foreign_key_target_is_decoded_before_the_naming_policies() {
    // The catalog spells a mixed-case target as SQL (`"Users"`); the policies see `Users`.
    let yaml = "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: last-word\n  \
                targetNames:\n    - tablePattern: '^Users$'\n      name: user\n";
    let table = |referenced: &str| {
        serde_json::json!({
            "formatVersion": 2, "coverage": "complete",
            "tables": { "lines": {
                "columns": { "user_id": { "dataType": "uuid" } },
                "foreignKeys": { "fk": {
                    "columns": ["user_id"], "referencedTable": referenced,
                    "referencedColumns": ["id"]
                } }
            } }
        })
    };
    assert!(super::support::findings(yaml, table("\"Users\"")).is_empty());
    // A schema-qualified target is matched as the decoded `schema.table`.
    let qualified = yaml.replace("'^Users$'", "'^other\\.Users$'");
    assert!(super::support::findings(&qualified, table("other.\"Users\"")).is_empty());
    // A name that only needs the quotes removed is singularized like an unquoted one.
    assert!(super::support::findings(yaml, table("\"users\"")).is_empty());
}
