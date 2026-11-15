use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;

fn encoding_marker(head: &str) -> bool {
    head == "invalid UTF-8 SQL file; save the file as UTF-8"
}

fn rejects_encoding_marker(file: &SqlStatementFileFacts) {
    assert!(
        file.parse_failures
            .iter()
            .all(|item| !encoding_marker(&item.head)),
        "{file:?}"
    );
}

#[test]
fn standalone_and_prepared_dml_decode_invalid_utf8_and_keep_valid_siblings() {
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
        &CheckFactPlan {
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
    rejects_encoding_marker(&facts.statements[0]);
    assert!(
        !facts.statements[0].parse_failed,
        "{:?}",
        facts.statements[0]
    );
    assert!(!facts.statements[1].parse_failed);
    let prepared = crate::codebase::postgres::prepared::prepare(
        &root,
        &files,
        &sources,
        &CheckFactPlan {
            postgres_dml: true,
            postgres_sql_include: vec!["*.sql".into()],
            ..Default::default()
        },
        &CheckFactMap::default(),
    );
    let bad = prepared.statements(&files[0], None).unwrap();
    rejects_encoding_marker(&bad[0]);
    assert!(!bad[0].parse_failed, "{bad:?}");
    assert!(!prepared.statements(&files[1], None).unwrap()[0].parse_failed);
}

#[test]
fn invalid_utf8_sql_keeps_an_offset_on_both_statement_passes() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/invalid-utf8-sql");
    let page = root.join("page.sql");
    let clean = root.join("clean.sql");
    let files = vec![page.clone(), clean.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = crate::codebase::postgres::collect_postgres_facts(
        &root,
        &sources,
        &files,
        &CheckFactPlan {
            postgres_dml: true,
            ..Default::default()
        },
        &crate::codebase::postgres::PostgresSchemaOptions {
            sql_include: vec!["*.sql".into()],
        },
        &crate::codebase::postgres::EmbeddedSqlOptions::configured("@db", &[]),
    )
    .unwrap();
    let page_facts = facts
        .statements
        .iter()
        .find(|file| file.path == page)
        .unwrap();
    rejects_encoding_marker(page_facts);
    assert!(
        page_facts.offset_uses.iter().any(|offset| offset.line == 2),
        "{page_facts:?}"
    );
    let clean_facts = facts
        .statements
        .iter()
        .find(|file| file.path == clean)
        .unwrap();
    assert!(clean_facts.offset_uses.is_empty(), "{clean_facts:?}");
    let prepared = crate::codebase::postgres::prepared::prepare(
        &root,
        &files,
        &sources,
        &CheckFactPlan {
            postgres_dml: true,
            postgres_sql_include: vec!["*.sql".into()],
            ..Default::default()
        },
        &CheckFactMap::default(),
    );
    let prepared_page = prepared.statements(&page, None).unwrap();
    rejects_encoding_marker(&prepared_page[0]);
    assert!(
        prepared_page[0]
            .offset_uses
            .iter()
            .any(|offset| offset.line == 2),
        "{prepared_page:?}"
    );
    assert!(prepared.statements(&clean, None).unwrap()[0]
        .offset_uses
        .is_empty());
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
        &CheckFactPlan {
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
