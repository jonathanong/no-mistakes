#[test]
fn verdict_recording_errors_when_embedded_path_has_no_statement_facts() {
    let root = fixture("fail");
    let listed = root.join("src/listed.ts");
    let embedded_path = root.join("src/embedded.ts");
    let opts = compile_options(&Options {
        extra_generated_columns: vec![ExtraGeneratedColumn {
            table: "orders".to_string(),
            column: "total".to_string(),
            source_column: "id".to_string(),
        }],
        ..Default::default()
    })
    .unwrap();
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.enable_sql_verdicts();
    let mut file = crate::codebase::check_facts::CheckFileFacts::default();
    file.embedded_sql.push((
        opts.embedded.clone(),
        crate::codebase::postgres::EmbeddedSqlFileFacts {
            path: embedded_path.clone(),
            executor_bindings: Vec::new(),
            calls: Vec::new(),
            call_spans: Vec::new(),
            exported_helpers: Default::default(),
            exported_constants: Default::default(),
            exported_programs: Default::default(),
            pending_imported: Vec::new(),
            imported_call_lines: Default::default(),
            source_comment_spans: Vec::new(),
            fragments: Vec::new(),
            fragment_sites: Vec::new(),
            fragment_variants: Vec::new(),
            matched_factory_names: Vec::new(),
            matched_type_names: Vec::new(),
            pending_relative: Default::default(),
            undeclared_executor_types: Vec::new(),
            unproven_member_query_spans: Vec::new(),
            postgres_rule_ids: Vec::new(),
            record_unproven_spans: true,
        },
    ));
    facts.ts.insert(listed.clone(), std::sync::Arc::new(file));
    facts.postgres.embedded.insert(
        (listed.clone(), opts.embedded.clone()),
        Ok(std::sync::Arc::new(Vec::new())),
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&listed));
    let error = scan::scan(
        &root,
        &opts,
        std::slice::from_ref(&listed),
        std::slice::from_ref(&embedded_path),
        &sources,
        &facts,
        &crate::codebase::rules::config_diagnostic::ConfigAnchor::fallback(),
    )
    .expect_err("statement facts");
    assert!(
        error
            .to_string()
            .contains("prepared PostgreSQL facts are missing"),
        "{error}"
    );
}
