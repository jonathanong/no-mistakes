use super::super::extract_migration_facts;

const COLUMN_SPECIFIC: &str = "\
ALTER TABLE child ADD CONSTRAINT child_topic_result_fk \
FOREIGN KEY (topic_id, result_id) REFERENCES parent (topic_id, result_id) \
ON DELETE SET NULL (result_id) NOT VALID;\n\
ALTER TABLE child VALIDATE CONSTRAINT child_topic_result_fk;";

#[test]
fn pairs_column_specific_set_null_and_keeps_the_action() {
    let facts = extract_migration_facts(COLUMN_SPECIFIC);
    assert_eq!(facts.not_valid_constraints.len(), 1, "{facts:?}");
    assert_eq!(facts.not_valid_constraints[0].name, "child_topic_result_fk");
    assert_eq!(facts.validated_constraints.len(), 1, "{facts:?}");
    assert_eq!(facts.validated_constraints[0].name, "child_topic_result_fk");
    assert_eq!(facts.foreign_keys.len(), 1, "{facts:?}");
    assert_eq!(
        facts.foreign_keys[0].column_names,
        ["topic_id", "result_id"]
    );
    assert_eq!(
        facts.foreign_keys[0].delete_action.as_deref(),
        Some("SET NULL")
    );
}

#[test]
fn pairs_ordinary_on_delete_set_null() {
    let facts = extract_migration_facts(
        "ALTER TABLE child ADD CONSTRAINT child_fk FOREIGN KEY (result_id) \
         REFERENCES parent (id) ON DELETE SET NULL NOT VALID;\n\
         ALTER TABLE child VALIDATE CONSTRAINT child_fk;",
    );
    assert_eq!(facts.not_valid_constraints[0].name, "child_fk");
    assert_eq!(facts.validated_constraints[0].name, "child_fk");
    assert_eq!(
        facts.foreign_keys[0].delete_action.as_deref(),
        Some("SET NULL")
    );
}

#[test]
fn keeps_unmatched_names_when_column_specific_set_null_parses() {
    let facts = extract_migration_facts(
        "ALTER TABLE child ADD CONSTRAINT child_topic_result_fk \
         FOREIGN KEY (topic_id, result_id) REFERENCES parent (topic_id, result_id) \
         ON DELETE SET NULL (result_id, topic_id) NOT VALID;\n\
         ALTER TABLE child VALIDATE CONSTRAINT child_other_fk;",
    );
    assert_eq!(facts.not_valid_constraints[0].name, "child_topic_result_fk");
    assert_eq!(facts.validated_constraints[0].name, "child_other_fk");
    assert_eq!(
        facts.foreign_keys[0].delete_action.as_deref(),
        Some("SET NULL")
    );
}

#[test]
fn pairs_column_specific_set_null_inside_do() {
    let facts = extract_migration_facts(
        "DO $$ BEGIN\n\
           ALTER TABLE child ADD CONSTRAINT child_topic_result_fk \
           FOREIGN KEY (result_id) REFERENCES parent (result_id) \
           ON DELETE SET NULL (result_id) NOT VALID;\n\
         END $$;\n\
         ALTER TABLE child VALIDATE CONSTRAINT child_topic_result_fk;",
    );
    assert_eq!(facts.not_valid_constraints[0].name, "child_topic_result_fk");
    assert_eq!(facts.validated_constraints[0].name, "child_topic_result_fk");
    assert_eq!(
        facts.foreign_keys[0].delete_action.as_deref(),
        Some("SET NULL")
    );
}

#[test]
fn create_table_column_specific_set_null_keeps_delete_action() {
    let facts = extract_migration_facts(
        "CREATE TABLE child (\n\
           result_id uuid REFERENCES parent (result_id) ON DELETE SET NULL (result_id)\n\
         );",
    );
    assert_eq!(facts.foreign_keys.len(), 1, "{facts:?}");
    assert_eq!(
        facts.foreign_keys[0].delete_action.as_deref(),
        Some("SET NULL")
    );
    assert!(facts.not_valid_constraints.is_empty());
}

#[test]
fn on_update_column_list_is_not_accepted_as_not_valid() {
    // PostgreSQL allows a SET NULL / SET DEFAULT column list only on ON DELETE.
    // Rewriting ON UPDATE would hide SQL the server rejects and pair the add.
    let facts = extract_migration_facts(
        "ALTER TABLE child ADD CONSTRAINT child_update_fk \
         FOREIGN KEY (result_id) REFERENCES parent (result_id) \
         ON UPDATE SET NULL (result_id) NOT VALID;\n\
         ALTER TABLE child VALIDATE CONSTRAINT child_update_fk;",
    );
    assert!(facts.not_valid_constraints.is_empty(), "{facts:?}");
    assert_eq!(facts.validated_constraints[0].name, "child_update_fk");
}

#[test]
fn delete_set_null_expression_is_not_accepted_as_not_valid() {
    // An expression is not a column list. Stripping it would hide SQL
    // PostgreSQL rejects and let the NOT VALID add pair.
    let facts = extract_migration_facts(
        "ALTER TABLE child ADD CONSTRAINT child_expr_fk \
         FOREIGN KEY (result_id) REFERENCES parent (result_id) \
         ON DELETE SET NULL (result_id + 1) NOT VALID;\n\
         ALTER TABLE child VALIDATE CONSTRAINT child_expr_fk;",
    );
    assert!(facts.not_valid_constraints.is_empty(), "{facts:?}");
    assert_eq!(facts.validated_constraints[0].name, "child_expr_fk");
}

#[test]
fn delete_set_null_reserved_word_is_not_accepted_as_not_valid() {
    // Unquoted NULL is reserved, so it is not a column name. A quoted
    // identifier would be valid; this form must not pair.
    let facts = extract_migration_facts(
        "ALTER TABLE child ADD CONSTRAINT child_null_fk \
         FOREIGN KEY (result_id) REFERENCES parent (result_id) \
         ON DELETE SET NULL (NULL) NOT VALID;\n\
         ALTER TABLE child VALIDATE CONSTRAINT child_null_fk;",
    );
    assert!(facts.not_valid_constraints.is_empty(), "{facts:?}");
    assert_eq!(facts.validated_constraints[0].name, "child_null_fk");
}

#[test]
fn column_default_expressions_stay_intact() {
    let facts = extract_migration_facts(
        "ALTER TABLE child ADD COLUMN result_id uuid DEFAULT (gen_random_uuid());",
    );
    assert_eq!(facts.add_columns.len(), 1, "{facts:?}");
    assert!(
        facts.add_columns[0]
            .default
            .as_deref()
            .unwrap_or("")
            .contains("gen_random_uuid"),
        "{facts:?}"
    );
}
