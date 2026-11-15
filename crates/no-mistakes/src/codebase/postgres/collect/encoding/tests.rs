fn encoding_marker(head: &str) -> bool {
    head == "invalid UTF-8 SQL file; save the file as UTF-8"
}

#[test]
fn standalone_schema_collection_decodes_invalid_utf8_and_keeps_valid_siblings() {
    use crate::codebase::postgres::extract_schema_facts;
    let root = crate::test_support::rule_fixture_root("postgres-regress-sql").join("encoding");
    let files = vec![root.join("bad.sql"), root.join("good.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = extract_schema_facts(&root, &sources, &files).unwrap();
    assert_eq!(facts.len(), 2);
    assert!(facts[0]
        .unanalyzable_statements
        .iter()
        .all(|item| !encoding_marker(&item.head)));
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
    assert!(facts.schema[0]
        .unanalyzable_statements
        .iter()
        .all(|item| !encoding_marker(&item.head)));
    assert_eq!(facts.schema[1].tables[0].table_name, "good");
    assert_eq!(facts.statements.len(), 2);
    assert!(facts.statements[0].parse_failed);
    assert_eq!(facts.statements[0].parse_failures.len(), 1);
    assert!(encoding_marker(&facts.statements[0].parse_failures[0].head));
    assert!(!facts.statements[1].parse_failed);
}

#[test]
fn invalid_utf8_schema_sql_keeps_foreign_keys_and_indexes() {
    use crate::codebase::postgres::extract_schema_facts;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules");
    let fk = root.join("postgres-fk-index/invalid-utf8/migrations/001.sql");
    let redundant = root.join("postgres-redundant-index/invalid-utf8/migrations/001.sql");
    let files = vec![fk.clone(), redundant.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = extract_schema_facts(&root, &sources, &files).unwrap();
    let fk_facts = facts.iter().find(|file| file.path == fk).unwrap();
    let index_facts = facts.iter().find(|file| file.path == redundant).unwrap();
    assert!(fk_facts.unanalyzable_statements.is_empty(), "{fk_facts:?}");
    assert!(fk_facts
        .foreign_keys
        .iter()
        .any(|key| { key.table_name == "comments" && key.column_names == ["post_id"] }));
    assert!(
        index_facts.unanalyzable_statements.is_empty(),
        "{index_facts:?}"
    );
    let names: Vec<_> = index_facts
        .indexes
        .iter()
        .filter_map(|index| index.name.as_deref())
        .collect();
    assert!(names.contains(&"idx_events__topic_id"), "{names:?}");
    assert!(
        names.contains(&"idx_events__topic_id__created_at"),
        "{names:?}"
    );
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
