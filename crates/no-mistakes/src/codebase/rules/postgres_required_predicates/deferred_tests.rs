use super::columns_tests::{config_yaml, fixture, messages, run};
use super::*;

const REQUIRED: &str =
    "sqlInclude: [sql/**/*.sql]\nrelations: [{table: orders, requireColumns: [account_id]}]\n";
const PARTITIONS: &str = "sqlInclude: [sql/**/*.sql]\nschemaCatalogPath: schema.json\npartitionKeys: require\npartitionKeyExemptions: [{table: mixed, reason: expression}, {table: missing, reason: stale snapshot}]\n";

#[test]
fn joins_apply_only_non_preserved_on_constraints_and_resolve_unaliased_self_join() {
    for (name, count) in [
        ("self-in", 1),
        ("outer-right", 1),
        ("outer-full", 1),
        ("inner-left", 0),
        ("self-unaliased", 0),
    ] {
        let findings = run("deferred", REQUIRED, &format!("sql/{name}.sql"));
        assert_eq!(findings.len(), count, "{name}: {findings:?}");
    }
}

#[test]
fn every_query_expression_and_values_body_is_walked() {
    let findings = run("deferred", REQUIRED, "sql/query-expressions.sql");
    assert_eq!(findings.len(), 4, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
}

#[test]
fn cte_scope_respects_recursion_and_quoted_case() {
    let findings = run("deferred", REQUIRED, "sql/ctes.sql");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 2);
}

#[test]
fn data_modifying_ctes_and_returning_reads_are_collected_once() {
    let findings = run("deferred", REQUIRED, "sql/cte-mutations.sql");
    assert_eq!(findings.len(), 5, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("UPDATE on"))
            .count(),
        2
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("DELETE on"))
            .count(),
        2
    );
    assert_eq!(run("deferred", REQUIRED, "sql/returning.sql").len(), 2);
}

#[test]
fn schema_identity_and_quoted_partition_columns_remain_distinct() {
    let qualified = run("deferred", PARTITIONS, "sql/qualified.sql");
    assert_eq!(qualified.len(), 2, "{qualified:?}");
    let text = messages(&qualified);
    assert!(text.contains("public.events"));
    assert!(text.contains("archive.events"));
    let quoted = run("deferred", PARTITIONS, "sql/quoted.sql");
    assert_eq!(quoted.len(), 1, "{quoted:?}");
    assert_eq!(quoted[0].line, 1);
    assert!(quoted[0].message.contains("TenantId"));
}

#[test]
fn mixed_keys_keep_columns_and_report_expressions_once_and_missing_keys_fail_closed() {
    let opts =
        "sqlInclude: [sql/**/*.sql]\nschemaCatalogPath: schema.json\npartitionKeys: require\n";
    let findings = run("deferred", opts, "sql/mixed.sql");
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("cannot derive"))
            .count(),
        1
    );
    assert!(messages(&findings).contains("partition key column account_id"));
    assert!(messages(&findings).contains("missing physicalPartition.key"));
    let allowed = run(
        "deferred",
        &format!("{opts}allow: [{{object: 'table:mixed', reason: reviewed}}]\n"),
        "sql/mixed.sql",
    );
    assert!(messages(&allowed).contains("partition key column account_id"));
    assert!(!messages(&allowed).contains("cannot derive"));
    let stale = run("deferred", &format!("{opts}partitionKeyExemptions: [{{table: absent, reason: reviewed}}]\nallow: [{{object: 'table:absent', reason: reviewed}}]\n"), "sql/mixed.sql");
    assert!(messages(&stale).contains("stale postgres-required-predicates partitionKeyExemptions"));
    assert!(messages(&stale).contains("schema.json: stale postgres-required-predicates allow"));
}

#[test]
fn relation_lines_are_per_occurrence_and_rebased_to_host_file() {
    let findings = run("deferred", REQUIRED, "sql/locations.sql");
    assert_eq!(findings[0].line, 3);
    let root = fixture("deferred");
    let file = root.join("queries.ts");
    let findings = check_with_files(
        &root,
        &config_yaml(&format!("{REQUIRED}importSpecifier: '@db'\n")),
        &[file],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 7, 9]
    );
}

#[test]
fn prepared_scans_reject_an_absent_request() {
    let root = fixture("deferred");
    let file = root.join("sql/self-in.sql");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    let opts =
        compile_options(&serde_yaml::from_str(&format!("{REQUIRED}executorNames: []\n")).unwrap())
            .unwrap();
    let error = scan::scan(&root, &opts, &[file], &sources, None).unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared PostgreSQL facts are required"));
}
