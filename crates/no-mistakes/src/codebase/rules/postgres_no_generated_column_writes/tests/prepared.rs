use super::*;

#[test]
fn do_writes_keep_physical_lines_and_suppressions() {
    let root = unit_fixture("do-write-lines");
    let paths = [root.join("schema.sql"), root.join("writes.sql")];
    let config = config_with_options("sqlInclude: [schema.sql]");
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &paths,
        sources.clone(),
        &config,
        &[RULE_ID],
    )
    .unwrap();
    let mut findings = check_with_files_sources_and_facts(&root, &config, &paths, &facts).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 6, 7, 8]
    );
    let source = sources.read_path(&paths[1]).unwrap();
    crate::codebase::rules::suppress_rule_findings_with_source(&mut findings, &source);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [7, 8]
    );
    assert_eq!(check_with_files(&root, &config, &paths).unwrap().len(), 4);
    assert!(std::ptr::eq(
        facts.postgres_statements(&paths[1], None).unwrap(),
        facts.postgres_statements(&paths[1], None).unwrap()
    ));
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn write_rule_projects_prepared_targets_without_reparsing() {
    let root = unit_fixture("do-write-lines");
    let paths = [root.join("schema.sql"), root.join("writes.sql")];
    let config = config_with_options("sqlInclude: [schema.sql]");
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let mut facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &paths,
        sources.clone(),
        &config,
        &[RULE_ID],
    )
    .unwrap();
    assert_eq!(
        check_with_files_sources_and_facts(&root, &config, &paths, &facts)
            .unwrap()
            .len(),
        4
    );
    // Keep the disk source unchanged: only a consumer that borrows prepared facts sees this.
    std::sync::Arc::make_mut(
        facts
            .postgres
            .statements
            .get_mut(&paths[1])
            .unwrap()
            .as_mut()
            .unwrap(),
    )[0]
    .writes
    .clear();
    assert!(
        check_with_files_sources_and_facts(&root, &config, &paths, &facts)
            .unwrap()
            .is_empty()
    );
    assert_eq!(sources.physical_read_count(), 2);
    facts.postgres.statements.insert(
        paths[1].clone(),
        Err(std::sync::Arc::new(
            crate::codebase::postgres::prepared::PreparationError {
                message: std::sync::Arc::from("recorded read failure"),
                source_kind: None,
            },
        )),
    );
    assert!(
        check_with_files_sources_and_facts(&root, &config, &paths, &facts)
            .unwrap()
            .is_empty()
    );
    facts.postgres.statements.remove(&paths[1]);
    assert!(check_with_files_sources_and_facts(&root, &config, &paths, &facts).is_err());
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn prepared_wildcard_and_non_sql_projection_remain_catalog_scoped() {
    let root = unit_fixture("do-write-lines");
    let non_sql = root.join("README.md");
    let paths = [root.join("schema.sql"), root.join("writes.sql"), non_sql];
    let config = config_with_options("sqlInclude: [schema.sql]\ninclude: ['**/*']");
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let mut facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &paths,
        sources,
        &config,
        &[RULE_ID],
    )
    .unwrap();
    let writes = &mut std::sync::Arc::make_mut(
        facts
            .postgres
            .statements
            .get_mut(&paths[1])
            .unwrap()
            .as_mut()
            .unwrap(),
    )[0]
    .writes;
    writes.truncate(1);
    writes[0].columns = crate::codebase::postgres::SqlWriteColumns::All;
    let findings = check_with_files_sources_and_facts(&root, &config, &paths, &facts).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("computed"));
}

#[test]
fn omitted_prepared_write_demand_is_an_error() {
    let root = unit_fixture("do-write-lines");
    let paths = [root.join("schema.sql"), root.join("writes.sql")];
    let error = check_with_files_sources_and_facts(
        &root,
        &config_with_options(""),
        &paths,
        &Default::default(),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared PostgreSQL facts are missing"));
}

#[test]
fn null_import_specifier_keeps_the_default_executor_profile() {
    let root = unit_fixture("quoted-identity");
    let paths = [root.join("schema.sql"), root.join("write.ts")];
    let findings =
        check_with_files(&root, &config_with_options("importSpecifier: null"), &paths).unwrap();
    assert_eq!(findings.len(), 2);
}
