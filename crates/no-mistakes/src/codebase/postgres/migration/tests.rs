use super::extract_migration_facts;

#[test]
fn extracts_indexes_foreign_keys_and_constraint_pairing() {
    let sql = r#"
CREATE TABLE comments (
  id uuid PRIMARY KEY,
  post_id uuid REFERENCES posts ON DELETE CASCADE
);
CREATE INDEX comments_post_id_idx ON comments (post_id);
CREATE INDEX comments_partial ON comments (author_id) WHERE author_id IS NOT NULL;
ALTER TABLE comments ADD CONSTRAINT comments_body_check CHECK (body <> '') NOT VALID;
ALTER TABLE comments VALIDATE CONSTRAINT comments_body_check;
"#;
    let facts = extract_migration_facts(sql);
    assert!(facts.indexes.iter().any(
        |index| index.table_name == "comments" && index.leading_column.as_deref() == Some("id")
    ));
    assert!(facts.indexes.iter().any(|index| {
        index.leading_column.as_deref() == Some("post_id") && index.access_method == "btree"
    }));
    assert!(facts.indexes.iter().any(|index| {
        index.leading_column.as_deref() == Some("author_id")
            && index.has_predicate
            && index.not_null_predicate_column.as_deref() == Some("author_id")
    }));
    assert_eq!(facts.foreign_keys.len(), 1);
    assert_eq!(facts.foreign_keys[0].column_names, ["post_id"]);
    assert_eq!(facts.foreign_keys[0].referenced_table_name, "posts");
    assert_eq!(
        facts.foreign_keys[0].delete_action.as_deref(),
        Some("CASCADE")
    );
    assert_eq!(facts.not_valid_constraints.len(), 1);
    assert_eq!(facts.not_valid_constraints[0].name, "comments_body_check");
    assert_eq!(facts.validated_constraints.len(), 1);
}

#[test]
fn extracts_alter_table_foreign_key() {
    let sql = "ALTER TABLE comments ADD CONSTRAINT comments_post_fk FOREIGN KEY (post_id) REFERENCES posts;";
    let facts = extract_migration_facts(sql);
    assert_eq!(facts.foreign_keys.len(), 1);
    assert_eq!(facts.foreign_keys[0].table_name, "comments");
    assert_eq!(facts.foreign_keys[0].column_names, ["post_id"]);
}

#[test]
fn extracts_table_level_keys_alter_shapes_and_index_predicates() {
    let sql = r#"
CREATE TABLE accounts (
  id uuid,
  email text,
  org_id uuid,
  UNIQUE (email),
  PRIMARY KEY (id),
  FOREIGN KEY (org_id) REFERENCES orgs
);
CREATE INDEX accounts_email_btree ON accounts USING btree (email);
CREATE INDEX accounts_org_hash ON accounts USING hash (org_id);
CREATE INDEX accounts_nested ON accounts (org_id) WHERE (org_id IS NOT NULL);
CREATE INDEX accounts_qualified ON accounts (org_id) WHERE accounts.org_id IS NOT NULL;
CREATE INDEX accounts_other ON accounts (org_id) WHERE org_id > 0;
CREATE INDEX accounts_literal ON accounts (org_id) WHERE 1 IS NOT NULL;
ALTER TABLE accounts ADD CONSTRAINT accounts_email_key UNIQUE (email);
ALTER TABLE accounts ADD CONSTRAINT accounts_pkey PRIMARY KEY (id);
ALTER TABLE accounts ADD CONSTRAINT accounts_org_fk FOREIGN KEY (org_id) REFERENCES orgs NOT VALID;
ALTER TABLE accounts ADD CONSTRAINT accounts_email_not_valid UNIQUE (email) NOT VALID;
ALTER TABLE accounts ADD CONSTRAINT accounts_pk_not_valid PRIMARY KEY (id) NOT VALID;
ALTER TABLE accounts ADD CONSTRAINT accounts_active CHECK (id IS NOT NULL);
ALTER TABLE accounts ADD COLUMN owner_id uuid REFERENCES users;
ALTER TABLE accounts DROP COLUMN unused;
"#;
    let facts = extract_migration_facts(sql);
    assert!(facts
        .indexes
        .iter()
        .any(|index| { index.leading_column.as_deref() == Some("email") && !index.has_predicate }));
    assert!(facts.indexes.iter().any(|index| {
        index.leading_column.as_deref() == Some("id") && index.access_method == "btree"
    }));
    assert!(facts
        .indexes
        .iter()
        .any(|index| index.access_method == "hash"));
    assert!(facts.indexes.iter().any(|index| {
        index.has_predicate && index.not_null_predicate_column.as_deref() == Some("org_id")
    }));
    assert!(facts
        .indexes
        .iter()
        .any(|index| index.has_predicate && index.not_null_predicate_column.is_none()));
    assert!(facts
        .foreign_keys
        .iter()
        .any(|fk| fk.column_names == ["org_id"] && fk.referenced_table_name == "orgs"));
    assert!(facts
        .foreign_keys
        .iter()
        .any(|fk| fk.column_names == ["owner_id"] && fk.referenced_table_name == "users"));
    assert!(facts
        .not_valid_constraints
        .iter()
        .any(|constraint| constraint.name == "accounts_org_fk"));
    assert!(facts
        .not_valid_constraints
        .iter()
        .any(|constraint| constraint.name == "accounts_email_not_valid"));
    assert!(facts
        .not_valid_constraints
        .iter()
        .any(|constraint| constraint.name == "accounts_pk_not_valid"));
}
