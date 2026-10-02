use super::tests::{config_yaml, fixture};
use super::*;

const SQL: &str = "schemaCatalogPath: schema.json\nsqlInclude: [sql/**/*.sql]\n";

fn run(file: &str, extra: &str) -> Vec<RuleFinding> {
    let root = fixture("deferred");
    check_with_files(
        &root,
        &config_yaml(&format!("{SQL}{extra}")),
        &[root.join(file)],
    )
    .unwrap()
}

#[test]
fn projection_spans_preserve_each_occurrence_and_suppression() {
    let findings = run("sql/lines.sql", "");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 6]
    );
    let root = fixture("deferred");
    for file in ["sql/suppress.sql", "suppressed.ts"] {
        let files = [root.join(file)];
        let mut findings = check_with_files(
            &root,
            &config_yaml(&format!("{SQL}importSpecifier: '@db'\n")),
            &files,
        )
        .unwrap();
        assert!(!findings.is_empty());
        super::super::suppress_rule_findings_with_sources(
            &root,
            &mut findings,
            &super::super::source_store_for_files(&files),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }
}

#[test]
fn qualified_identities_and_cte_names_follow_postgres_semantics() {
    let qualified = run("sql/qualified.sql", "");
    assert_eq!(qualified.len(), 2, "{qualified:?}");
    assert_eq!(
        qualified
            .iter()
            .map(|finding| finding.target.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["public.orders", "archive.orders"]
    );
    let ctes = run("sql/ctes.sql", "");
    assert_eq!(ctes.len(), 1, "{ctes:?}");
    assert_eq!(ctes[0].line, 2);
    let listed = run(
        "sql/qualified.sql",
        "maxColumns: 4294967295\nrelations: [public.orders]\n",
    );
    assert_eq!(listed.len(), 1);
    assert!(listed[0].message.contains("configured in relations"));
}

#[test]
fn cte_mutations_and_returning_peers_include_all_visible_relations() {
    let findings = run("sql/dml-ctes.sql", "");
    assert_eq!(findings.len(), 5, "{findings:?}");
    assert!(findings.iter().all(|finding| finding
        .message
        .contains("RETURNING * returns all 40 columns of orders")));
    assert!(run("sql/dml-ctes.sql", "checkReturning: false\n").is_empty());
    let peers = run("sql/peers.sql", "");
    assert_eq!(peers.len(), 3, "{peers:?}");
    assert_eq!(
        peers.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        [1, 2, 4]
    );
}

#[test]
fn expression_visitors_find_whole_rows_and_values_reads_but_skip_table_functions() {
    let functions = run("sql/functions.sql", "allowWholeRowFunctions: []\n");
    assert_eq!(functions.len(), 2, "{functions:?}");
    assert_eq!(
        functions
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    assert!(run("sql/functions.sql", "").is_empty());
    assert_eq!(run("sql/values.sql", "").len(), 1);
}

#[test]
fn parse_and_dynamic_failures_use_host_lines_and_honor_ignore() {
    let findings = run("queries.ts", "importSpecifier: '@db'\n");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 5, 7, 10]
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("SQL could not be analyzed"))
            .count(),
        2
    );
    assert!(findings[3].message.contains("not statically recoverable"));
    let ignored = run(
        "queries.ts",
        "importSpecifier: '@db'\nunanalyzableSql: ignore\n",
    );
    assert_eq!(ignored.len(), 1);
    assert_eq!(run("sql/invalid.sql", "").len(), 1);
    assert!(run("sql/invalid.sql", "unanalyzableSql: ignore\n").is_empty());
}

#[test]
fn known_views_are_reported_only_when_explicitly_listed() {
    assert!(run("sql/view.sql", "maxColumns: 0\n").is_empty());
    let findings = run(
        "sql/view.sql",
        "maxColumns: 0\nrelations: [view_public_tags]\n",
    );
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("configured in relations"));
}

#[test]
fn overflowing_thresholds_are_configuration_errors() {
    let opts: Options =
        serde_yaml::from_str("schemaCatalogPath: schema.json\nmaxColumns: 4294967296\n").unwrap();
    assert!(compile_options(&opts)
        .err()
        .unwrap()
        .to_string()
        .contains("must be at most 4294967295"));
}

#[test]
fn prepared_scans_fail_on_absent_fact_demand_and_keep_sql_opt_in() {
    let root = fixture("deferred");
    let file = root.join("sql/lines.sql");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    let opts = compile_options(&serde_yaml::from_str(SQL).unwrap()).unwrap();
    assert!(
        scan::scan(&root, &opts, std::slice::from_ref(&file), &sources, None)
            .unwrap_err()
            .to_string()
            .contains("prepared PostgreSQL facts are required")
    );
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    facts
        .postgres_schema_catalogs
        .insert("schema.json".into(), Ok(std::sync::Arc::new(catalog)));
    assert!(scan::scan(
        &root,
        &opts,
        std::slice::from_ref(&file),
        &sources,
        Some(&facts)
    )
    .unwrap_err()
    .to_string()
    .contains("prepared PostgreSQL facts are missing"));
    let config = config_yaml("schemaCatalogPath: schema.json\n");
    let prepared = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &[file],
        sources,
        &config,
        &[RULE_ID],
    )
    .unwrap();
    assert!(prepared.postgres.statements.is_empty());
}

#[test]
fn insert_aliases_and_default_values_expand_the_physical_target() {
    let findings = run("sql/insert-alias.sql", "");
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(findings[0].message.contains("RETURNING *"));
    assert!(findings[1].message.contains("RETURNING *"));
    assert!(findings[2].target.as_deref() == Some("public.orders"));
}

#[test]
fn multiline_whole_rows_and_lenient_projections_keep_suppression_lines() {
    let root = fixture("deferred");
    for (file, expected) in [("function-lines.sql", 3), ("lenient-lines.sql", 4)] {
        let files = [root.join("sql").join(file)];
        let mut findings = check_with_files(
            &root,
            &config_yaml(&format!(
                "{SQL}allowWholeRowFunctions: []\nunanalyzableSql: ignore\n"
            )),
            &files,
        )
        .unwrap();
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].line, expected);
        super::super::suppress_rule_findings_with_sources(
            &root,
            &mut findings,
            &super::super::source_store_for_files(&files),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }
}
