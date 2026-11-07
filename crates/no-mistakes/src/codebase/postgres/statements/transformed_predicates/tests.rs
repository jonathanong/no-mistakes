use crate::codebase::postgres::{extract_sql_statement_facts, SqlStatementFileFacts};
#[path = "cast_sites_tests.rs"]
mod cast_sites_tests;

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
    assert_eq!(lines, [1, 1, 2, 3, 4, 5, 6, 7]);
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

#[test]
fn recovered_transform_binds_preserve_real_columns_and_quoted_marker_names() {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-no-transformed-uuid-predicates")
            .join("recovered-transform-bind.sql"),
    )
    .unwrap();
    let column = source.find("sql_placeholder_1").unwrap() as u32 + 1;
    let facts = super::super::extract_sql_statement_facts_with_recovered_placeholders(
        &source,
        true,
        &[(1, column)],
    );
    assert!(!facts.parse_failed);
    let columns: Vec<_> = facts
        .transformed_column_uses
        .iter()
        .map(|use_| (use_.line, use_.column.as_str()))
        .collect();
    assert_eq!(
        columns,
        [(1, "token"), (2, "sql_placeholder_1"), (2, "token")]
    );
    assert_eq!(
        super::super::extract_sql_statement_facts(&source)
            .transformed_column_uses
            .len(),
        4
    );
}

#[test]
fn expression_sites_group_arguments_and_nested_calls_without_merging_siblings() {
    let facts = facts("multi-argument-sites.sql");
    let uses = &facts.transformed_column_uses;
    for (line, count) in [(1, 2), (2, 4), (3, 8)] {
        let arguments: Vec<_> = uses.iter().filter(|use_| use_.line == line).collect();
        assert_eq!(arguments.len(), count);
        assert!(arguments
            .iter()
            .all(|use_| use_.expression_site == arguments[0].expression_site));
    }
    let siblings: std::collections::BTreeSet<_> = uses
        .iter()
        .filter(|use_| use_.line == 4)
        .map(|use_| use_.expression_site)
        .collect();
    assert_eq!(siblings.len(), 2);
    let multiline: Vec<_> = uses
        .iter()
        .filter(|use_| [6, 7].contains(&use_.line))
        .collect();
    assert_eq!(multiline.len(), 2);
    assert_eq!(multiline[0].expression_site, multiline[1].expression_site);
    assert_eq!(multiline[0].expression_site.0, 5);
}

#[test]
fn cast_sites_use_operator_tokens_without_inheriting_boolean_or_subquery_wrappers() {
    let facts = facts("cast-site-controls.sql");
    let sites: Vec<_> = facts
        .transformed_column_uses
        .iter()
        .map(|use_| use_.expression_site)
        .collect();
    assert_eq!(
        sites,
        [
            (1, 30),
            (2, 38),
            (3, 30),
            (4, 46),
            (5, 35),
            (6, 37),
            (7, 45),
            (8, 43),
            (10, 35),
            (10, 35),
        ]
    );
}
