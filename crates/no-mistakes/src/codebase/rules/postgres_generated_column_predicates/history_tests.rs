use super::tests::{config_yaml, fixture};
use super::*;
const SQL: &str = "sqlInclude: ['sql/**/*.sql']\n";

#[test]
fn mutation_history_applies_alter_drops_and_recreation_in_order() {
    let root = fixture("history-followups");
    let config = config_yaml(SQL);
    let create = root.join("sql/001-create.sql");
    let alter = root.join("sql/002-alter.sql");
    let query = root.join("sql/query.sql");
    assert_eq!(
        check_with_files(
            &root,
            &config,
            &[create.clone(), alter.clone(), query.clone()]
        )
        .unwrap()
        .len(),
        2
    );
    let dropped = [
        create.clone(),
        alter,
        root.join("sql/003-drop.sql"),
        query.clone(),
    ];
    assert!(check_with_files(&root, &config, &dropped)
        .unwrap()
        .is_empty());
    let recreated = [
        create,
        root.join("sql/003-drop.sql"),
        root.join("sql/004-recreate.sql"),
        query.clone(),
    ];
    assert!(check_with_files(&root, &config, &recreated)
        .unwrap()
        .is_empty());
    assert!(
        check_with_files(&root, &config, &[root.join("sql/same-file.sql"), query])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn supplied_request_facts_are_reused_for_repeated_rule_consumers() {
    let root = fixture("prepared-reuse");
    let files = vec![root.join("sql/schema.sql"), root.join("src/query.ts")];
    let config = config_yaml(&format!("{SQL}include: ['src/**/*.ts']\n"));
    let sources = super::super::source_store_for_files(&files);
    crate::ast::begin_parse_count(&root);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        std::sync::Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap();
    let schema = facts.postgres_schema_file(&files[0]).unwrap() as *const _;
    let first =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    let second =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&files[1]), Some(&1), "{counts:?}");
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert_eq!(
        schema,
        facts.postgres_schema_file(&files[0]).unwrap() as *const _
    );
}

#[test]
fn prepared_failure_entries_are_returned_without_fallback_collection() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
    let root = fixture("review-followups");
    let config = config_yaml(SQL);
    for paths in [
        vec![root.join("sql/missing.sql")],
        vec![root.join("src/missing.ts")],
    ] {
        assert!(check_with_files(&root, &config, &paths).is_err());
    }
    for paths in [
        vec![root.join("sql/schema.sql")],
        vec![root.join("src/query.ts")],
    ] {
        let sources = super::super::source_store_for_files(&paths);
        let plan = CheckFactPlan {
            embedded_sql: true,
            embedded_sql_options: vec![crate::codebase::postgres::EmbeddedSqlOptions::configured(
                "@example/db",
                &[],
            )],
            ..Default::default()
        };
        let facts = collect_check_facts(&root, paths.clone(), plan);
        assert!(
            check_with_files_sources_and_facts(&root, &config, &paths, &sources, &facts).is_err()
        );
    }
}

#[test]
fn request_config_validation_rejects_invalid_options_before_scanning() {
    let root = fixture("review-followups");
    let paths = [root.join("src/query.ts")];
    for yaml in [
        "executorNames: query",
        "include: 12",
        "include: ['[']",
        "exclude: ['[']",
        "sqlInclude: ['[']",
        "functions: []",
    ] {
        assert!(
            check_with_files(&root, &config_yaml(yaml), &paths).is_err(),
            "{yaml}"
        );
    }
}
