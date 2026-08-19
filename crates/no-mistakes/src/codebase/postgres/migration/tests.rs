use super::extract_migration_facts;

#[test]
fn extracts_indexes_foreign_keys_and_constraint_pairing() {
    let sql = r#"
CREATE TABLE comments (
  id uuid PRIMARY KEY,
  post_id uuid REFERENCES posts
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
