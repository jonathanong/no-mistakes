use super::extract_dml_write_targets;

#[test]
fn extracts_update_insert_and_merge_tables() {
    let sql = r#"
        UPDATE public.items SET note = 'x';
        INSERT INTO ONLY "MixedCase" VALUES (1);
        MERGE INTO other.t AS dest USING s ON true WHEN MATCHED THEN UPDATE SET n = 1;
    "#;
    assert_eq!(extract_dml_write_targets(sql), ["MixedCase", "items", "t"]);
}

#[test]
fn skips_update_set_without_a_table() {
    let sql = "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET note = 1";
    assert_eq!(extract_dml_write_targets(sql), ["items"]);
}

#[test]
fn ignores_select_and_empty_input() {
    assert!(extract_dml_write_targets("SELECT created_at FROM items").is_empty());
    assert!(extract_dml_write_targets("").is_empty());
}

#[test]
fn last_identifier_wins_for_schema_qualified_names() {
    assert_eq!(
        extract_dml_write_targets(r#"UPDATE "public"."Votes" SET n = 1"#),
        ["Votes"]
    );
    assert_eq!(
        extract_dml_write_targets("insert into only votes values (1)"),
        ["votes"]
    );
    assert!(extract_dml_write_targets(r#"UPDATE "" SET n = 1"#).is_empty());
}
