use crate::codebase::postgres::{extract_sql_statement_facts, SqlStatementFileFacts};

fn facts(name: &str) -> SqlStatementFileFacts {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-no-transformed-uuid-predicates")
            .join(name),
    )
    .unwrap();
    let facts = extract_sql_statement_facts(&source);
    assert!(!facts.parse_failed, "{facts:#?}");
    facts
}

#[test]
fn aggregates_are_not_scalar_column_transforms_in_the_shared_statement_facts() {
    let facts = facts("aggregate-operands-pass.sql");
    assert!(facts.transformed_column_uses.is_empty(), "{facts:#?}");
}

#[test]
fn aggregate_boundaries_preserve_filter_and_subquery_predicates_with_locations() {
    let facts = facts("aggregate-operands-positive.sql");
    assert_eq!(
        facts
            .transformed_column_uses
            .iter()
            .map(|fact| (
                fact.line,
                fact.table.as_str(),
                fact.column.as_str(),
                fact.transformation.as_str()
            ))
            .collect::<Vec<_>>(),
        [
            (2, "", "id", "cast"),
            (3, "", "id", "fn"),
            (4, "", "id", "fn"),
            (5, "", "id", "app.count"),
            (6, "", "id", "app.min"),
            (7, "", "id", "fn"),
            (8, "", "id", "cast"),
            (9, "", "token", "fn"),
            (10, "", "token", "fn"),
            (11, "", "token", "COALESCE"),
            (12, "", "id", "fn"),
            (13, "", "email", "fn"),
            (14, "", "id", "\"MIN\""),
            (15, "accounts", "id", "cast"),
            (16, "", "id", "cast"),
            (17, "", "id", "fn"),
        ]
    );
}

#[test]
fn driving_values_leave_the_joined_key_bare_but_residuals_remain_checked() {
    let facts = facts("probe-direction.sql");
    let lines: Vec<_> = facts
        .transformed_column_uses
        .iter()
        .map(|fact| fact.line)
        .collect();
    assert_eq!(
        lines,
        [
            3, 3, 4, 4, 6, 6, 7, 7, 8, 8, 9, 9, 9, 10, 10, 11, 12, 13, 14, 15, 15, 19, 19, 20, 20,
            21, 21, 22, 22, 25, 27, 28, 29, 30, 30, 31, 31
        ]
    );
}
