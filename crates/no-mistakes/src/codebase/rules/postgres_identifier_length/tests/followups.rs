use super::*;

#[test]
fn schema_only_preparation_keeps_embedded_typescript_out_of_scope() {
    let sql = unit("max-bytes.sql");
    let ts = unit("ignored.ts");
    let paths = [sql.clone(), ts.clone()];
    let root = sql.parent().unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let config = config("maxBytes: 10\n");
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        &paths,
        std::sync::Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap();
    assert!(facts.postgres_schema_file(&sql).is_ok());
    assert!(facts.postgres_statements(&sql, None).is_err());
    assert!(facts
        .postgres_statements(&ts, Some(&Default::default()))
        .is_err());
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn configured_schema_rules_borrow_one_prepared_migration_projection() {
    let path = unit("declaration-locations.sql");
    let root = path.parent().unwrap();
    let paths = [path.clone()];
    let mut config = config("maxBytes: 10\n");
    config.rules.push(RuleDef {
        rule: "postgres-no-add-column".into(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        &paths,
        std::sync::Arc::clone(&sources),
        &config,
        &[RULE_ID, "postgres-no-add-column"],
    )
    .unwrap();
    let before = facts.postgres_schema_file(&path).unwrap();
    let findings =
        check_with_files_sources_and_facts(root, &config, &paths, &sources, &facts).unwrap();
    assert!(!findings.is_empty());
    assert!(
        crate::codebase::rules::postgres_no_add_column::check_with_files_sources_and_facts(
            root, &config, &paths, &sources, &facts
        )
        .unwrap()
        .is_empty()
    );
    assert!(std::ptr::eq(
        before,
        facts.postgres_schema_file(&path).unwrap()
    ));
    // Prepared consumers must not silently fall back to a second collection.
    assert!(check_with_files_sources_and_facts(
        root,
        &config,
        &paths,
        &sources,
        &crate::codebase::check_facts::CheckFactMap::default()
    )
    .unwrap_err()
    .to_string()
    .contains("prepared PostgreSQL facts are missing"));
}
