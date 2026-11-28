use crate::statements::{
    extract_sql_statement_facts_with_policy, SqlLocationDemand, SqlStatementFileFacts,
    StatementPolicySources,
};

fn fixture_facts(fixture: &str) -> SqlStatementFileFacts {
    let root = crate::test_support::rule_fixture_root("postgres-no-duplicate-keyed-lookups");
    let sql = std::fs::read_to_string(root.join(fixture).join("queries/check.sql")).unwrap();
    extract_sql_statement_facts_with_policy(
        &sql,
        false,
        None,
        SqlLocationDemand::None,
        StatementPolicySources {
            collect_duplicate_keyed_lookups: true,
            ..Default::default()
        },
        false,
    )
}

#[test]
fn distinguishes_projection_ordering_and_normalizes_qualified_sort_keys() {
    let facts = fixture_facts("normalization");

    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.relation.as_str(), fact.first_line, fact.line))
        .collect();
    assert_eq!(sites, [("accounts", 16, 24), ("accounts", 44, 45)]);
}

#[test]
fn detects_duplicates_within_query_scopes() {
    let facts = fixture_facts("query-scopes");

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

#[test]
fn constrains_duplicates_to_equivalent_lookup_scopes_and_row_sets() {
    let facts = fixture_facts("review-soundness");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [(2, 3), (9, 10), (14, 15), (20, 21), (55, 55), (68, 69)]
    );
}

#[test]
fn ignores_reused_target_expressions_and_reports_distinct_sort_lookups() {
    let facts = fixture_facts("target-list");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(sites, [(19, 21)]);
}

#[test]
fn detects_executed_clause_lookups_without_colliding_quoted_arguments() {
    let facts = fixture_facts("query-clauses");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (3, 4),
            (9, 10),
            (15, 16),
            (21, 22),
            (27, 28),
            (33, 34),
            (40, 41),
            (47, 49),
            (63, 64)
        ]
    );
}

#[test]
fn resolves_function_and_join_scopes_without_merging_distinct_predicates() {
    let facts = fixture_facts("review-edges");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (8, 9),
            (17, 18),
            (23, 24),
            (29, 30),
            (34, 35),
            (43, 44),
            (49, 50),
            (55, 57),
            (61, 63),
            (67, 68)
        ]
    );
}

#[test]
fn distinguishes_evaluated_order_and_limit_lookups_from_unproven_operations() {
    let facts = fixture_facts("review-planning");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(sites, [(3, 4), (10, 11), (29, 30), (33, 34), (61, 62)]);
}

#[test]
fn skips_only_incomplete_statements_during_lookup_extraction() {
    let root = crate::test_support::rule_fixture_root("postgres-no-duplicate-keyed-lookups")
        .join("statement-completeness");
    let sql = std::fs::read_to_string(root.join("queries/check.sql")).unwrap();
    let complete: Vec<bool> =
        serde_json::from_slice(&std::fs::read(root.join("complete.json")).unwrap()).unwrap();
    let prepared = crate::parse::PreparedSql::new(&sql);
    let statements = prepared.parse_policy().unwrap();
    assert_eq!(statements.len(), complete.len());
    let facts = crate::statements::extract_from_parsed_with_recovered_placeholders(
        &sql,
        &prepared,
        &statements,
        true,
        false,
        None,
        StatementPolicySources {
            collect_duplicate_keyed_lookups: true,
            statement_complete: Some(&complete),
            ..Default::default()
        },
    );
    let lines: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| fact.line)
        .collect();
    assert!(facts.parse_failed);
    assert_eq!(lines, [1, 3]);
}

#[test]
fn proves_using_bindings_and_distinguishes_reused_window_targets_from_frame_bounds() {
    let facts = fixture_facts("review-window-using");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(sites, [(19, 20), (32, 33), (40, 41), (71, 72)]);
}

#[test]
fn shares_lookup_history_for_unnest_and_query_limits_and_checks_projection_purity() {
    let facts = fixture_facts("review-shared-query");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (3, 4),
            (8, 9),
            (14, 15),
            (21, 22),
            (28, 29),
            (51, 52),
            (56, 57)
        ]
    );
}

#[test]
fn reuses_resolved_grouping_and_order_targets_and_visits_distinct_window_references() {
    let facts = fixture_facts("review-target-reuse");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (16, 17),
            (24, 26),
            (32, 33),
            (43, 44),
            (47, 48),
            (59, 60),
            (65, 66),
            (73, 74),
            (80, 81),
            (88, 89),
            (125, 126),
            (135, 136)
        ]
    );
}

#[test]
fn retains_lookup_identity_across_sorts_merge_categories_and_reused_targets() {
    let facts = fixture_facts("review-clause-identity");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [(3, 6), (18, 19), (24, 25), (31, 32), (47, 48), (64, 65)]
    );
}

#[test]
fn separates_grouped_reuse_from_independent_expressions_and_case_arms() {
    let facts = fixture_facts("review-grouping-case");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (8, 9),
            (13, 14),
            (26, 27),
            (35, 36),
            (41, 43),
            (47, 49),
            (61, 62),
            (67, 68),
            (72, 75),
            (88, 89),
            (95, 96),
            (133, 130),
            (129, 131)
        ]
    );
}

#[test]
fn respects_null_extended_bindings_and_reuses_aliased_targets_and_window_specs() {
    let facts = fixture_facts("review-join-window");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (5, 6),
            (12, 13),
            (20, 22),
            (27, 28),
            (41, 42),
            (51, 53),
            (57, 58)
        ]
    );
}

#[test]
fn checks_proven_operator_inputs_and_query_binding_and_evaluation_order() {
    let facts = fixture_facts("review-binding-evaluation");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [(8, 9), (23, 24), (28, 29), (40, 42), (45, 46), (51, 52)]
    );
}

#[test]
fn respects_scalar_grouping_nested_sort_defaults_and_eager_window_inputs() {
    let facts = fixture_facts("review-evaluation-followup");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(sites, [(8, 9), (23, 24), (43, 44)]);
}

#[test]
fn preserves_sql_target_identity_and_proves_equality_predicate_inputs() {
    let facts = fixture_facts("review-sql-identity");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (13, 14),
            (28, 29),
            (33, 37),
            (46, 47),
            (58, 59),
            (67, 68),
            (72, 73),
            (80, 81),
            (87, 88),
            (93, 94),
            (98, 99)
        ]
    );
}

#[test]
fn checks_operator_signatures_and_shares_only_compatible_lookup_history() {
    let facts = fixture_facts("review-operator-filter");
    assert!(!facts.parse_failed);
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();
    assert_eq!(
        sites,
        [
            (17, 18),
            (31, 32),
            (36, 37),
            (49, 51),
            (55, 63),
            (88, 89),
            (98, 99),
            (116, 117),
            (127, 128),
            (144, 146),
            (158, 170),
            (195, 201)
        ]
    );
}
