use super::shape;

#[test]
fn standalone_table_queries_emit_bounded_facts() {
    let unaffected = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-nonquery-control.sql"
    ));
    assert_eq!(shape(unaffected), ["select: accounts"]);

    let bare = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-bare.sql"
    ));
    assert_eq!(shape(bare), ["select: accounts"]);

    let with_ddl = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-temporary.sql"
    ));
    assert_eq!(
        shape(with_ddl),
        [
            "select: opaque",
            "select: accounts",
            "select: public.\"Accounts\"",
            "select: public.accounts",
            "select: opaque",
            "select: opaque"
        ]
    );

    let recovered = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-lenient.sql"
    ));
    assert_eq!(shape(recovered), ["select: opaque", "select: accounts"]);
}

#[test]
fn invalid_standalone_table_clause_stays_unanalyzable() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-invalid-where.sql"
    ));
    assert!(super::extract_sql_statement_facts(sql).parse_failed);
    let missing_names = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-invalid-names.sql"
    ));
    assert!(super::extract_sql_statement_facts(missing_names).parse_failed);
}

#[test]
fn quoted_standalone_arm_keeps_its_source_spelling_before_select_from() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-set-arms.sql"
    ));
    assert!(
        crate::codebase::postgres::parse_postgres_sql(sql).is_ok(),
        "{:?}",
        crate::codebase::postgres::parse_postgres_sql(sql).err()
    );
    assert_eq!(shape(sql), ["select: (opaque) (accounts)"]);
}

#[test]
fn repeated_standalone_table_facts_keep_distinct_lines() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-lines.sql"
    ));
    let facts = super::extract_sql_statement_facts(sql);
    assert_eq!(
        facts
            .selects
            .iter()
            .map(|select| select.line)
            .collect::<Vec<_>>(),
        [2, 3]
    );
}

#[test]
fn standalone_table_locking_clauses_remain_queries() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-locking.sql"
    ));
    assert!(!super::extract_sql_statement_facts(sql).parse_failed);
    assert_eq!(shape(sql), ["select: accounts", "select: accounts"]);
}

#[test]
fn standalone_table_explicit_descendants_remain_queries() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-inheritance-star.sql"
    ));
    assert!(
        crate::codebase::postgres::parse_postgres_sql(sql).is_ok(),
        "{:?}",
        crate::codebase::postgres::parse_postgres_sql(sql).err()
    );
    assert_eq!(shape(sql), ["select: accounts", "select: accounts"]);
}
