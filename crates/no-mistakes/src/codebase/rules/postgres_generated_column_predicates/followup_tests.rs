use super::tests::{config_yaml, fixture};
use super::*;

const SQL: &str = "sqlInclude: ['sql/**/*.sql']\n";

#[test]
fn predicates_keep_locations_relation_scopes_and_schema_identities() {
    let root = fixture("review-followups");
    for (name, lines) in [
        ("lines", vec![3, 4]),
        ("inequality", vec![1]),
        ("alias", vec![3]),
        ("mutations", vec![2, 4, 5]),
        ("alter", vec![1]),
        ("qualified", vec![1]),
        ("join-scope", vec![2]),
        ("unknown", vec![]),
    ] {
        let files = [
            root.join("sql/schema.sql"),
            root.join(format!("sql/{name}.sql")),
        ];
        let findings = check_with_files(&root, &config_yaml(SQL), &files).unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            lines,
            "{name}: {findings:#?}"
        );
        assert!(findings
            .iter()
            .all(|finding| !finding.message.contains("UUIDv7 bound")
                && !finding.message.contains("primary-key index")));
    }
}

#[test]
fn schema_inputs_survive_query_includes_and_all_js_ts_extensions_work() {
    let root = fixture("review-followups");
    for ext in ["ts", "mjs", "cjs", "jsx", "cts"] {
        let files = [
            root.join("sql/schema.sql"),
            root.join(format!("src/query.{ext}")),
        ];
        for yaml in [
            SQL.to_string(),
            format!("{SQL}include: ['src/**/*.{ext}']\n"),
        ] {
            let findings = check_with_files(&root, &config_yaml(&yaml), &files).unwrap();
            assert_eq!(findings.len(), 1, "{ext}: {findings:#?}");
            assert_eq!(findings[0].line, 2);
        }
    }
}

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
fn each_clause_can_be_disabled_with_real_join_inputs() {
    let root = fixture("review-followups");
    let files = [
        root.join("sql/schema.sql"),
        root.join("sql/join-scope.sql"),
        root.join("sql/lines.sql"),
    ];
    let findings = check_with_files(
        &root,
        &config_yaml(&format!("{SQL}clauses: [where, order-by]\n")),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 2);
    assert!(!findings
        .iter()
        .any(|finding| finding.message.contains("JOIN ON compares")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("WHERE filters")));
}

#[test]
fn extra_columns_work_without_a_schema_definition() {
    let root = fixture("review-followups");
    let config = config_yaml("extraGeneratedColumns: [{table: events, column: created_at, sourceColumn: id}]\nfunctions: [uuid_extract_timestamp, other_fn]\n");
    let findings = check_with_files(&root, &config, &[root.join("sql/extra.sql")]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0]
        .message
        .contains("WHERE filters events.created_at"));
    assert!(!findings[0].message.contains("uuid_extract_timestamp(id)"));
}

#[test]
fn parse_errors_and_suppressions_use_host_and_expression_lines() {
    let root = fixture("review-followups");
    let schema = root.join("sql/schema.sql");
    let parse = root.join("src/parse.ts");
    let findings = check_with_files(&root, &config_yaml(SQL), &[schema.clone(), parse]).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 4);
    for name in ["sql/suppress.sql", "src/suppress.ts"] {
        let files = [schema.clone(), root.join(name)];
        let mut findings = check_with_files(&root, &config_yaml(SQL), &files).unwrap();
        assert!(!findings.is_empty());
        let sources = super::super::source_store_for_files(&files);
        super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
        assert!(findings.is_empty(), "{findings:#?}");
    }
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
fn arbitrary_generation_functions_do_not_claim_uuid_or_primary_key_semantics() {
    let root = fixture("review-followups");
    let config = config_yaml(&format!(
        "{SQL}functions: [other_fn]\nrequireArgumentIsPrimaryKey: false\n"
    ));
    let findings = check_with_files(
        &root,
        &config,
        &[
            root.join("sql/custom-schema.sql"),
            root.join("sql/custom-query.sql"),
        ],
    )
    .unwrap();
    assert_eq!(findings.len(), 2, "{findings:#?}");
    for finding in findings {
        assert!(finding.message.contains("other_fn(slug)"));
        assert!(!finding.message.contains("UUIDv7"));
        assert!(!finding.message.contains("same order"));
        assert!(!finding.message.contains("primary-key index"));
        assert!(finding
            .message
            .contains("preserves the generation expression's semantics"));
    }
}

#[test]
fn quoted_names_keep_case_and_dynamic_calls_report_when_schema_is_tracked() {
    let root = fixture("review-followups");
    let paths = [
        root.join("sql/quoted-schema.sql"),
        root.join("sql/schema.sql"),
        root.join("sql/quoted.sql"),
    ];
    let findings = check_with_files(&root, &config_yaml(SQL), &paths).unwrap();
    assert_eq!(findings.iter().map(|f| f.line).collect::<Vec<_>>(), [1, 2]);
    let paths = [
        root.join("sql/schema.sql"),
        root.join("src/dynamic.ts"),
        root.join("sql/missing-owner.sql"),
    ];
    let findings = check_with_files(&root, &config_yaml(SQL), &paths).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("not statically recoverable"));
}
