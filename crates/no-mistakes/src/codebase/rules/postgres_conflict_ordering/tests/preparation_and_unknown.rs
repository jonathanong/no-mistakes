#[test]
fn requires_a_catalog_path() {
    let error = match compile_options(&Options::default()) {
        Ok(_) => panic!("missing schemaCatalogPath should fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("schemaCatalogPath"));
}

#[test]
fn compile_options_canonicalize_executor_names_for_prepared_fact_lookup() {
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        executor_names: vec![
            "write".to_string(),
            "query".to_string(),
            "write".to_string(),
        ],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.embedded.executor_names, ["query", "write"]);
}

#[test]
fn prepared_scan_contextualizes_a_missing_embedded_sql_projection() {
    let root = fixture("pass-canonical-order");
    let file = root.join("src/insert.ts");
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&file),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.postgres_schema_catalogs.insert(
        "schema.json".to_string(),
        Ok(std::sync::Arc::new(
            crate::codebase::postgres::SchemaCatalog::default(),
        )),
    );
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        ..Options::default()
    })
    .unwrap();

    let error = scan::scan_with_sources(
        &root,
        &compiled,
        std::slice::from_ref(&file),
        &sources,
        &facts,
    )
    .expect_err("missing prepared facts must fail closed");
    assert!(error.to_string().contains(
        "postgres-conflict-ordering failed to collect embedded SQL facts from prepared analysis"
    ));
    assert!(format!("{error:#}").contains(&file.display().to_string()));
}

#[test]
fn rejects_opaque_executor_arguments_by_default() {
    let findings = findings("fail-opaque-executor");
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(
        findings
            .iter()
            .all(|finding| finding.target.as_deref() == Some("unanalyzable-sql")),
        "{findings:#?}"
    );
}

#[test]
fn ignores_opaque_executor_arguments_when_unanalyzable_sql_is_ignore() {
    let root = fixture("fail-opaque-executor");
    let result = check_with_files(
        &root,
        &config_with_options("schemaCatalogPath: schema.json\nunanalyzableSql: ignore"),
        &files(&root),
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn rejects_opaque_member_query_arguments_by_default() {
    let findings = findings("fail-opaque-member-query");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn rejects_recovered_dynamic_conflict_sql_by_default() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[root.join("src/dynamic.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert_eq!(result.len(), 1, "{result:#?}");
    assert_eq!(result[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn rejects_a_dynamic_insert_before_its_conflict_clause_is_recovered() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[
            root.join("src/dynamic-insert-fragment.ts"),
            root.join("schema.json"),
        ],
    )
    .unwrap();
    assert_eq!(result.len(), 1, "{result:#?}");
    assert_eq!(result[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn ignores_recovered_dynamic_non_insert_sql() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[root.join("src/dynamic-select.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn ignores_dynamic_non_insert_sql_that_only_mentions_insert_in_comments_or_strings() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[
            root.join("src/dynamic-comment-and-string.ts"),
            root.join("schema.json"),
        ],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn allows_an_explicit_unanalyzable_sql_exception() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config_with_options("schemaCatalogPath: schema.json\nunanalyzableSql: ignore"),
        &[root.join("src/dynamic.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn honors_the_safe_directive_for_recovered_dynamic_sql() {
    let root = fixture("pass-canonical-order");
    let result = check_with_files(
        &root,
        &config(),
        &[root.join("src/dynamic-safe.ts"), root.join("schema.json")],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}
