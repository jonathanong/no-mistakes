use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn recursive_table_arms_report_exact_catalog_leaves() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    for fixture in [
        "recursive-table-forward.sql",
        "recursive-table-quoted-union.sql",
        "recursive-table-qualified-union.sql",
    ] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{fixture} must parse as one statement");
        assert_eq!(facts.bounds.len(), 1, "{fixture}");
    }
}
