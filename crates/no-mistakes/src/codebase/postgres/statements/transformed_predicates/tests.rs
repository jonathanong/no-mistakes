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
fn driving_values_retain_probe_metadata_for_catalog_classification() {
    let facts = facts("probe-direction.sql");
    let probes: Vec<_> = facts
        .transformed_column_uses
        .iter()
        .filter(|fact| matches!(fact.line, 1 | 32 | 33 | 34))
        .map(|fact| {
            let probe = fact
                .driving_probe
                .as_ref()
                .expect("catalog proof remains required");
            (fact.line, probe.table.as_str(), probe.column.as_str())
        })
        .collect();
    assert_eq!(
        probes,
        [
            (1, "accounts", "id"),
            (1, "accounts", "id"),
            (32, "missing", "id"),
            (32, "missing", "id"),
            (33, "accounts", "missing"),
            (33, "accounts", "missing"),
            (34, "accounts", "email"),
            (34, "accounts", "email"),
        ]
    );
}

#[test]
fn null_tests_ignore_scalar_transforms_but_retain_nested_predicates() {
    assert!(facts("probe-followup-null-pass.sql")
        .transformed_column_uses
        .is_empty());
    let facts = facts("probe-followup-null-positive.sql");
    let lines: Vec<_> = facts
        .transformed_column_uses
        .iter()
        .map(|fact| fact.line)
        .collect();
    assert_eq!(lines, [1, 1, 2, 3, 4, 5, 6]);
}

#[test]
fn distinct_bare_operands_retain_catalog_probe_candidates_in_both_directions() {
    let pass = facts("probe-followup-distinct-pass.sql");
    assert_eq!(pass.transformed_column_uses.len(), 8);
    for fact in pass.transformed_column_uses {
        let probe = fact.driving_probe.expect("bare opposite operand");
        assert_eq!(probe.column, "id");
        assert_eq!(probe.candidate_tables, Some(vec!["accounts".into()]));
    }
    let facts = facts("probe-followup-distinct-positive.sql");
    assert!(facts
        .transformed_column_uses
        .iter()
        .filter(|fact| fact.line <= 6)
        .all(|fact| fact
            .driving_probe
            .as_ref()
            .is_none_or(|probe| probe.column == "email")));
}
