#[test]
fn repeated_scans_match() {
    let fail = [
        "sql/schema.sql",
        "sql/where.sql",
        "sql/order.sql",
        "sql/between.sql",
    ];
    assert_eq!(messages("fail", SQL, &fail), messages("fail", SQL, &fail));
}

#[test]
fn an_extra_column_absent_from_the_schema_is_tracked() {
    let body = messages(
        "pass",
        "sqlInclude: [\"sql/**/*.sql\"]\nextraGeneratedColumns:\n  - {table: invoices, column: created_at, sourceColumn: id}\n",
        &["sql/schema.sql", "sql/invoices.sql"],
    )
    .join("\n");
    assert!(body.contains("ORDER BY invoices.created_at"), "{body}");
}

#[test]
fn an_explicit_include_skips_other_files() {
    let body = messages(
        "fail",
        "include: [\"sql/schema.sql\", \"sql/where.sql\"]\nsqlInclude: [\"sql/**/*.sql\"]\n",
        &["sql/schema.sql", "sql/where.sql", "sql/order.sql"],
    )
    .join("\n");
    assert!(body.contains("WHERE filters"), "{body}");
    assert!(!body.contains("ORDER BY"), "{body}");
}

#[test]
fn no_tracked_columns_are_quiet_even_with_dynamic_sql() {
    let root = fixture("coverage");
    let file = root.join("src/query.ts");
    assert!(
        check_with_files(&root, &config_yaml("include: ['src/**/*.ts']\n"), &[file])
            .unwrap()
            .is_empty()
    );
}

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
    )
    .expect_err("statement facts");
    assert!(
        error
            .to_string()
            .contains("prepared PostgreSQL facts are missing"),
        "{error}"
    );
}

#[test]
fn suppression_directives_hide_predicates() {
    let root = fixture("suppress");
    let files: Vec<PathBuf> = ["schema.sql", "next-line.sql", "line.sql", "file.sql"]
        .into_iter()
        .map(|name| root.join("sql").join(name))
        .collect();
    let mut findings = check_with_files(&root, &config_yaml(SQL), &files).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}
