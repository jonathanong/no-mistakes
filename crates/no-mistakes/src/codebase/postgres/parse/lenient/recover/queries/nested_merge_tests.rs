use crate::codebase::postgres::migration::extract_migration_facts;
use std::path::PathBuf;

#[test]
fn nested_row_locked_loop_with_first_action_merge_recovers_action() {
    let sql = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/ddl-compat/body-recovery/query-loop-matrix")
            .join("nested-locked-merge.sql"),
    )
    .unwrap();

    let facts = extract_migration_facts(&sql);
    assert!(facts.unanalyzable_statements.is_empty(), "{facts:#?}");
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.name_parts == ["abs"] && call.line == 10));
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "following_query_loop"));
}
