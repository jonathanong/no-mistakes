use crate::statements::{
    extract_sql_statement_facts_with_policy, SqlLocationDemand, StatementPolicySources,
};

#[test]
fn distinguishes_projection_ordering_and_normalizes_qualified_sort_keys() {
    let root = crate::test_support::rule_fixture_root("postgres-no-duplicate-keyed-lookups");
    let sql = std::fs::read_to_string(root.join("normalization/queries/check.sql")).unwrap();
    let facts = extract_sql_statement_facts_with_policy(
        &sql,
        false,
        None,
        SqlLocationDemand::None,
        StatementPolicySources {
            collect_duplicate_keyed_lookups: true,
            ..Default::default()
        },
        false,
    );

    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.relation.as_str(), fact.first_line, fact.line))
        .collect();
    assert_eq!(sites, [("accounts", 16, 24), ("accounts", 44, 45)]);
}

#[test]
fn detects_duplicates_within_query_scopes() {
    let root = crate::test_support::rule_fixture_root("postgres-no-duplicate-keyed-lookups");
    let sql = std::fs::read_to_string(root.join("query-scopes/queries/check.sql")).unwrap();
    let facts = extract_sql_statement_facts_with_policy(
        &sql,
        false,
        None,
        SqlLocationDemand::None,
        StatementPolicySources {
            collect_duplicate_keyed_lookups: true,
            ..Default::default()
        },
        false,
    );

    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (4, 5),
            (10, 11),
            (15, 16),
            (20, 21),
            (26, 27),
            (39, 40),
            (46, 47),
            (52, 53)
        ]
    );
}
