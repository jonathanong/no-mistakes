use super::{blocking_statuses, bound_body_observed};
use crate::codebase::postgres::statements::bounds::Scope;
use sqlparser::ast::Statement;

#[test]
fn long_mixed_union_chain_checks_each_set_node_once() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/long-mixed-set-operation.sql"),
    )
    .unwrap();
    let statements = crate::codebase::postgres::parse_postgres_sql(&sql).unwrap();
    let Statement::Query(query) = &statements[0] else {
        panic!("expected one query");
    };
    let mut visits = 0;
    let bound = bound_body_observed(query, &Scope::default(), None, |blocking| {
        visits = blocking.visits;
    });
    assert!(!bound.capped, "the blocking first arm must remain uncapped");
    // There are 32 UNION ALL arms plus the blocking first arm. Count both
    // initial status evaluations and capping lookups; rescanning each left
    // spine during capping would require hundreds more visits.
    assert!(
        visits <= 160,
        "blocking-status visits during bounding and capping: {visits}",
    );
    assert!(
        visits > 64,
        "the test must traverse and cap the long set chain"
    );
}

#[test]
fn all_streaming_union_chain_skips_status_storage() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/all-streaming-set-operation.sql"),
    )
    .unwrap();
    let statements = crate::codebase::postgres::parse_postgres_sql(&sql).unwrap();
    let Statement::Query(query) = &statements[0] else {
        panic!("expected one query");
    };
    assert!(blocking_statuses(&query.body).is_none());
}
