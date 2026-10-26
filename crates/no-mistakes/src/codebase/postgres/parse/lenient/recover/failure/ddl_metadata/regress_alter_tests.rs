use crate::codebase::postgres::migration::extract_migration_facts;

#[test]
fn regression_alter_forms_are_parsed_or_recognized() {
    let valid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-alter-valid.sql"
    ));
    let missing = valid
        .lines()
        .filter(|sql| {
            extract_migration_facts(sql)
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported)
        })
        .collect::<Vec<_>>();
    assert!(missing.is_empty(), "{missing:#?}");
}

#[test]
fn regression_alter_malformed_tails_remain_failures() {
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-alter-invalid.sql"
    ));
    for sql in invalid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}: {:?}",
            facts.unanalyzable_statements
        );
    }
}

#[test]
fn descendant_marker_preserves_add_column_fact() {
    // The marker may be removed only after validating the original name position;
    // downstream policy still needs the ADD COLUMN fact.
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-alter-valid.sql"
    ))
    .lines()
    .find(|line| line.starts_with("ALTER TABLE a_star*"))
    .unwrap();
    let facts = extract_migration_facts(sql);
    assert_eq!(facts.add_columns.len(), 1);
    assert_eq!(facts.add_columns[0].table_name, "a_star");
    assert_eq!(facts.add_columns[0].column_name, "a");
}
