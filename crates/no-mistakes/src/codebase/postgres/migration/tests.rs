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
        index.name.as_deref() == Some("comments_post_id_idx")
            && index.leading_column.as_deref() == Some("post_id")
            && index.access_method == "btree"
            && !index.unique
            && index.columns.len() == 1
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
        .add_columns
        .iter()
        .any(|column| { column.table_name == "accounts" && column.column_name == "owner_id" }));
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

#[test]
fn index_table_constraints_do_not_contribute_not_valid_names() {
    use sqlparser::ast::{IndexConstraint, TableConstraint};
    let constraint = TableConstraint::Index(IndexConstraint {
        display_as_key: false,
        name: Some(sqlparser::ast::Ident::new("idx")),
        index_type: None,
        columns: Vec::new(),
        index_options: Vec::new(),
    });
    assert!(super::constraints::constraint_name(&constraint).is_none());
}

#[test]
fn extracts_not_valid_and_fk_index_from_do_block() {
    let sql = r#"
DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'fk_posts_community_id' AND conrelid = 'posts'::regclass
  ) THEN
    ALTER TABLE posts ADD CONSTRAINT fk_posts_community_id
      FOREIGN KEY (community_id) REFERENCES communities(id)
      ON DELETE SET NULL NOT VALID;
  END IF;
END $$;
ALTER TABLE posts VALIDATE CONSTRAINT fk_posts_community_id;
DO $$ BEGIN
  CREATE INDEX idx_posts__community_id ON posts (community_id);
END $$;
"#;
    let facts = extract_migration_facts(sql);
    assert!(
        facts
            .not_valid_constraints
            .iter()
            .any(|constraint| constraint.name == "fk_posts_community_id"),
        "{facts:?}"
    );
    assert!(
        facts
            .validated_constraints
            .iter()
            .any(|constraint| constraint.name == "fk_posts_community_id"),
        "{facts:?}"
    );
    assert!(
        facts
            .foreign_keys
            .iter()
            .any(|fk| fk.column_names == ["community_id"] && fk.table_name == "posts"),
        "{facts:?}"
    );
    assert!(
        facts.indexes.iter().any(|index| {
            index.table_name == "posts" && index.leading_column.as_deref() == Some("community_id")
        }),
        "{facts:?}"
    );
}

#[test]
fn extracted_schema_records_clone_eq_and_debug() {
    let facts = extract_migration_facts(
        "CREATE TABLE t (id uuid PRIMARY KEY, other_id uuid REFERENCES other);\n\
         ALTER TABLE t ADD CONSTRAINT t_id_check CHECK (id IS NOT NULL) NOT VALID;\n\
         ALTER TABLE t VALIDATE CONSTRAINT t_id_check;\n",
    );
    let indexes = facts.indexes.clone();
    let foreign_keys = facts.foreign_keys.clone();
    let not_valid = facts.not_valid_constraints.clone();
    let validated = facts.validated_constraints.clone();
    assert_eq!(indexes, facts.indexes);
    assert_eq!(foreign_keys, facts.foreign_keys);
    assert_eq!(not_valid, facts.not_valid_constraints);
    assert_eq!(validated, facts.validated_constraints);
    assert!(format!("{indexes:?}{foreign_keys:?}{not_valid:?}{validated:?}").contains("t"));
}

#[test]
fn extracts_index_prefix_fields_includes_and_drops() {
    let sql = r#"
CREATE UNIQUE INDEX idx_accounts__email ON accounts (email DESC NULLS LAST) INCLUDE (region);
CREATE INDEX idx_accounts__org ON accounts (org_id) WHERE deleted_at IS NULL;
DROP INDEX IF EXISTS public.first_index, second_index;
"#;
    let facts = extract_migration_facts(sql);
    let unique = facts
        .indexes
        .iter()
        .find(|index| index.name.as_deref() == Some("idx_accounts__email"))
        .expect("unique");
    assert!(unique.unique);
    assert_eq!(unique.include_columns, ["region"]);
    assert_eq!(unique.columns[0].name.as_deref(), Some("email"));
    assert_eq!(unique.columns[0].ordering.as_deref(), Some("desc"));
    assert_eq!(unique.columns[0].nulls_ordering.as_deref(), Some("last"));
    let partial = facts
        .indexes
        .iter()
        .find(|index| index.name.as_deref() == Some("idx_accounts__org"))
        .expect("partial");
    assert_eq!(partial.predicate_key.as_deref(), Some("deleted_at is null"));
    assert_eq!(
        facts
            .dropped_indexes
            .iter()
            .map(|drop| drop.name.as_str())
            .collect::<Vec<_>>(),
        ["public.first_index", "second_index"]
    );
    assert!(facts.dropped_indexes.iter().all(|drop| drop.line > 0));
    let ordered = extract_migration_facts(
        "CREATE INDEX idx_asc ON t (email ASC NULLS FIRST);\n\
         CREATE INDEX idx_gin ON t USING gin (email);",
    );
    let asc = ordered
        .indexes
        .iter()
        .find(|index| index.name.as_deref() == Some("idx_asc"))
        .expect("asc");
    assert_eq!(asc.columns[0].ordering.as_deref(), Some("asc"));
    assert_eq!(asc.columns[0].nulls_ordering.as_deref(), Some("first"));
    assert!(ordered
        .indexes
        .iter()
        .any(|index| index.name.as_deref() == Some("idx_gin") && index.access_method == "gin"));
}

#[test]
fn preserves_schema_qualified_index_identities_and_drop_tables() {
    let sql = r#"
CREATE INDEX idx ON public.events (topic_id);
CREATE INDEX idx ON audit.events (topic_id);
DROP INDEX audit.idx;
DROP TABLE public.events;
"#;
    let facts = extract_migration_facts(sql);
    assert_eq!(
        facts
            .indexes
            .iter()
            .map(|index| (index.table_name.as_str(), index.name.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("public.events", Some("idx")),
            ("audit.events", Some("idx"))
        ]
    );
    assert_eq!(
        facts
            .dropped_indexes
            .iter()
            .map(|drop| drop.name.as_str())
            .collect::<Vec<_>>(),
        ["audit.idx"]
    );
    assert_eq!(
        facts
            .dropped_tables
            .iter()
            .map(|drop| drop.name.as_str())
            .collect::<Vec<_>>(),
        ["public.events"]
    );
}

#[test]
fn predicate_keys_preserve_literal_and_quoted_ident_case() {
    let sql = r#"
CREATE INDEX idx_active ON events (topic_id) WHERE status = 'ACTIVE';
CREATE INDEX idx_active_lower ON events (topic_id) WHERE status = 'active';
CREATE INDEX idx_quoted ON events (topic_id) WHERE "Status" IS NOT NULL;
"#;
    let facts = extract_migration_facts(sql);
    let keys: Vec<_> = facts
        .indexes
        .iter()
        .filter_map(|index| index.predicate_key.as_deref())
        .collect();
    assert_eq!(
        keys,
        [
            "status = 'ACTIVE'",
            "status = 'active'",
            "\"Status\" is not null"
        ]
    );
}

#[test]
fn multiline_create_index_line_matches_the_create_keyword() {
    let sql = "SELECT 1;\nCREATE INDEX\n  idx_events__topic_id\n  ON events (topic_id);";
    let facts = extract_migration_facts(sql);
    assert_eq!(facts.indexes[0].line, 2);
    assert_eq!(
        facts.indexes[0].name.as_deref(),
        Some("idx_events__topic_id")
    );
}

#[test]
fn covering_indexes_keep_schema_qualifiers() {
    let sql = "CREATE TABLE public.accounts (id uuid PRIMARY KEY);";
    let facts = extract_migration_facts(sql);
    assert!(facts
        .indexes
        .iter()
        .any(|index| index.table_name == "public.accounts" && index.unique));
    let alter = extract_migration_facts(
        "ALTER TABLE public.accounts ADD CONSTRAINT accounts_email_key UNIQUE (email);",
    );
    assert!(alter
        .indexes
        .iter()
        .any(|index| index.table_name == "public.accounts" && index.unique));
}

#[test]
fn qualified_relation_joins_identifiers_and_skips_functions() {
    use sqlparser::ast::{Ident, ObjectName, ObjectNamePart};
    let name = ObjectName(vec![
        ObjectNamePart::Identifier(Ident::new("public")),
        ObjectNamePart::Identifier(Ident::new("events")),
    ]);
    assert_eq!(super::qualified_relation(&name), "public.events");
    assert_eq!(super::qualified_relation(&ObjectName(Vec::new())), "");
    let function = ObjectName(vec![ObjectNamePart::Function(
        sqlparser::ast::ObjectNamePartFunction {
            name: Ident::new("fn"),
            args: Vec::new(),
        },
    )]);
    assert_eq!(super::qualified_relation(&function), "");
}

#[test]
fn extracts_add_column_inside_do_block() {
    let facts =
        extract_migration_facts("DO $$ BEGIN ALTER TABLE posts ADD COLUMN status text; END $$;");
    assert!(facts
        .add_columns
        .iter()
        .any(|column| column.table_name == "posts" && column.column_name == "status"));
}

#[test]
fn records_unnamed_alter_table_fk_and_check() {
    let facts = extract_migration_facts(
        "ALTER TABLE children ADD CHECK (parent_id IS NOT NULL) NOT VALID;\n\
         ALTER TABLE children ADD FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE NOT VALID;\n\
         ALTER TABLE children ADD CONSTRAINT fk_named FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE;\n\
         ALTER TABLE children ADD UNIQUE (parent_id);\n",
    );
    assert_eq!(
        facts
            .unnamed_constraints
            .iter()
            .map(|constraint| constraint.kind.as_str())
            .collect::<Vec<_>>(),
        ["CHECK", "FOREIGN KEY"],
        "{facts:?}"
    );
    assert!(facts
        .unnamed_constraints
        .iter()
        .all(|constraint| constraint.table_name == "children"));
}
