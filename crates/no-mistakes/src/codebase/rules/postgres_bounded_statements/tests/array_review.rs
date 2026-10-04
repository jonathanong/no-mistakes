use super::names;

#[test]
fn finite_array_constructors_require_bounded_scalar_catalog_leaves() {
    let root = super::fixture_root();
    let path = root.join("schema-finite-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema-finite-array.json", &sources)
            .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/finite-array.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [
            5, 6, 7, 8, 10, 11, 12, 13, 15, 16, 17, 18, 20, 27, 28, 29, 30, 33, 35, 36, 40, 43, 46,
            49, 52, 53, 54, 79, 80, 81
        ]
        .map(|line| ("accounts".to_string(), line))
    );
}

#[test]
fn indexed_array_elements_require_scalar_catalog_elements_and_bounded_owners() {
    let root = super::fixture_root();
    let path = root.join("schema-signed-network-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog = crate::codebase::postgres::SchemaCatalog::load(
        &root,
        "schema-signed-network-array.json",
        &sources,
    )
    .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/indexed-array.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [
            ("accounts", 3),
            ("accounts", 4),
            ("accounts", 7),
            ("accounts", 8),
            ("accounts", 9),
            ("accounts", 10),
            ("accounts", 11),
            ("accounts", 15),
            ("accounts", 16),
            ("accounts", 17),
            ("accounts", 19)
        ]
        .map(|(name, line)| (name.to_string(), line))
    );
}

#[test]
fn signed_literals_and_builtin_network_columns_keep_finite_array_pins() {
    let root = super::fixture_root();
    let path = root.join("schema-signed-network-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog = crate::codebase::postgres::SchemaCatalog::load(
        &root,
        "schema-signed-network-array.json",
        &sources,
    )
    .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/signed-network-array.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.line)
        .collect();
    assert_eq!(found, [3, 4, 5, 13, 14, 15, 16, 17, 18, 19, 20, 23, 30]);
}

#[test]
fn stored_arrays_do_not_inherit_their_rows_bound() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/stored-array.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts"]);
}

#[test]
fn enum_casts_and_interval_literals_preserve_finite_array_cardinality() {
    let root = super::fixture_root();
    let path = root.join("schema-signed-network-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog = crate::codebase::postgres::SchemaCatalog::load(
        &root,
        "schema-signed-network-array.json",
        &sources,
    )
    .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/literal-array-types.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.line)
        .collect();
    assert_eq!(found, [3, 4, 9]);
}
