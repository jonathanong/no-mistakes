use super::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

fn fixture_root() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("unique-index-shapes")
}

pub(super) fn observed(sql_file: &str) -> Vec<Vec<String>> {
    observed_with_catalog(sql_file, "schema.json")
}

pub(super) fn observed_with_catalog(sql_file: &str, catalog_file: &str) -> Vec<Vec<String>> {
    let root = fixture_root();
    let catalog =
        SchemaCatalog::from_json(&std::fs::read_to_string(root.join(catalog_file)).unwrap())
            .unwrap();
    let sql = std::fs::read_to_string(root.join(sql_file)).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{sql_file}: {facts:#?}");
    facts
        .bounds
        .iter()
        .map(|fact| {
            offenders(fact, &catalog)
                .into_iter()
                .map(|finding| finding.table)
                .collect()
        })
        .collect()
}

#[test]
fn partial_unique_keys_need_an_implied_predicate() {
    assert_eq!(
        observed("partial-unique.sql"),
        [
            vec![],
            vec!["partial_docs"],
            vec!["partial_docs"],
            vec!["partial_docs"],
            vec!["partial_docs"],
            vec!["partial_docs"],
        ]
    );
}

#[test]
fn expression_unique_keys_match_equality_and_bound_joins() {
    assert_eq!(
        observed("expression-unique.sql"),
        [
            vec![],
            vec![],
            vec!["normalized_emails"],
            vec!["normalized_emails"],
            vec![],
            vec!["normalized_emails"],
        ]
    );
}

#[test]
fn partial_expression_keys_require_a_null_guard_when_equality_does_not_prove_it() {
    assert_eq!(
        observed("partial-expression-unique.sql"),
        [
            vec!["partial_emails"],
            vec![],
            vec!["partial_emails"],
            vec!["partial_emails"],
            vec!["partial_emails"],
        ]
    );
}

#[test]
fn equality_implies_a_partial_column_null_guard_but_null_safe_equality_does_not() {
    assert_eq!(
        observed("partial-column-nullguard.sql"),
        [vec![], vec!["partial_logins"]]
    );
}

#[test]
fn null_safe_equality_pins_only_nulls_not_distinct_unique_keys() {
    assert_eq!(
        observed("nulls-not-distinct.sql"),
        [vec![], vec!["plain_contacts"], vec![]]
    );
}
