use super::shape_tests::{config_yaml, fixture, BANNED};
use super::*;

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

#[test]
fn cooked_and_delayed_sql_initializers_use_physical_source_lines() {
    let root = fixture("review-followups");
    let path = root.join("src/source-map.ts");
    let mut findings =
        check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&path)).unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [2, 6, 8, 10, 12]
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [2, 6, 8, 10]
    );
}

#[test]
fn comment_and_operator_suppressions_use_actual_expression_lines() {
    let root = fixture("review-followups");
    let file = root.join("sql/suppress.sql");
    let mut findings =
        check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 2);
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn filter_facts_are_unique_and_lenient_spans_preserve_suppression() {
    let root = fixture("review-followups");
    let sql = std::fs::read_to_string(root.join("sql/filter-once.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert_eq!(
        facts
            .selects
            .iter()
            .map(|s| s.not_in_subqueries.len())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        facts
            .selects
            .iter()
            .map(|s| s.count_existence_checks.len())
            .sum::<usize>(),
        1
    );
    let path = root.join("sql/lenient-locations.sql");
    let config = config_yaml(&format!("{BANNED}\nunanalyzableSql: ignore"));
    let mut findings = check_with_files(&root, &config, std::slice::from_ref(&path)).unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [3, 4],
        "{findings:#?}"
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(findings.iter().map(|f| f.line).collect::<Vec<_>>(), [4]);
}
