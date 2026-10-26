#[test]
fn standalone_schema_collection_retains_bad_encoding_and_valid_siblings() {
    use crate::codebase::postgres::extract_schema_facts;
    let root = crate::test_support::rule_fixture_root("postgres-regress-sql").join("encoding");
    let files = vec![root.join("bad.sql"), root.join("good.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = extract_schema_facts(&root, &sources, &files).unwrap();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].unanalyzable_statements.len(), 1);
    assert!(!facts[0].unanalyzable_statements[0].valid_unsupported);
    assert_eq!(facts[1].tables[0].table_name, "good");
    let facts = crate::codebase::postgres::collect_postgres_facts(
        &root,
        &sources,
        &files,
        &crate::codebase::check_facts::CheckFactPlan {
            postgres_schema: true,
            postgres_dml: true,
            ..Default::default()
        },
        &crate::codebase::postgres::PostgresSchemaOptions {
            sql_include: vec!["*.sql".into()],
        },
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("@db", &[]),
    )
    .unwrap();
    assert_eq!(facts.statements.len(), 2);
    assert!(facts.statements[0].parse_failed);
    assert_eq!(facts.statements[0].parse_failures.len(), 1);
    assert!(!facts.statements[1].parse_failed);
}

#[test]
fn required_missing_sql_retains_its_read_error() {
    let root = crate::test_support::rule_fixture_root("postgres-regress-sql").join("encoding");
    let files = vec![root.join("intentionally-absent.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let error = match crate::codebase::postgres::collect_postgres_facts(
        &root,
        &sources,
        &files,
        &crate::codebase::check_facts::CheckFactPlan {
            postgres_dml: true,
            ..Default::default()
        },
        &crate::codebase::postgres::PostgresSchemaOptions {
            sql_include: vec!["*.sql".into()],
        },
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("@db", &[]),
    ) {
        Ok(_) => panic!("a missing required input remains an error"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("failed to read"));
}
