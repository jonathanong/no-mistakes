use super::*;

#[test]
fn conflict_function_facts_are_shared_without_schema_duplicates() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/clause-policy"),
    );
    let sql = root.join("review.sql");
    let files = vec![sql.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let plan = CheckFactPlan {
        postgres_schema: true,
        postgres_dml: true,
        postgres_sql_include: vec!["**/*.sql".into()],
        ..Default::default()
    };
    let facts = prepare(&root, &files, &sources, &plan, &CheckFactMap::default());
    let schema = facts.schema(&sql).unwrap();
    let statements = &facts.statements(&sql, None).unwrap()[0];
    assert!(!statements.parse_failed);
    assert_eq!(schema.function_calls.len(), 7);
    assert_eq!(schema.function_calls, statements.function_calls);
    let source = sources.read_path(&sql).unwrap();
    let standalone = crate::codebase::postgres::extract_sql_statement_facts(&source);
    assert_eq!(standalone.function_calls, statements.function_calls);
    let facts_without_schema = prepare(
        &root,
        &files,
        &sources,
        &CheckFactPlan {
            postgres_schema: false,
            ..plan
        },
        &CheckFactMap::default(),
    );
    assert_eq!(
        facts_without_schema.statements(&sql, None).unwrap()[0].function_calls,
        statements.function_calls
    );
}
