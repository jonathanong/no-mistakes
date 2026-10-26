use super::unique_index_shapes::observed;

#[test]
fn partial_keys_propagate_only_where_the_predicate_restricts_the_scan() {
    assert_eq!(
        observed("joins-and-conjunctions.sql"),
        [
            vec![],
            vec!["community_members"],
            vec![],
            vec!["community_members"],
            vec!["community_members"],
            vec![],
            vec![],
            vec!["partial_multi"],
            vec!["partial_multi"],
            vec!["partial_docs"],
        ]
    );
}

#[test]
fn null_safe_expression_pins_require_nulls_not_distinct() {
    assert_eq!(
        observed("nullsafe-expressions.sql"),
        [vec![], vec!["normalized_emails"]]
    );
}

#[test]
fn aliases_and_expression_null_tests_keep_their_catalog_meaning() {
    assert_eq!(
        observed("index-guards.sql"),
        [
            vec!["nd_contacts"],
            vec![],
            vec!["normalized_emails"],
            vec!["partial_expr_nullguard"],
            vec![]
        ]
    );
}

#[test]
fn malformed_catalog_predicates_and_keys_never_establish_bounds() {
    assert_eq!(
        super::unique_index_shapes::observed_with_catalog(
            "guards/malformed.sql",
            "guards/schema.json"
        ),
        [vec!["broken_predicate"], vec!["broken_expression"]]
    );
}
