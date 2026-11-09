use super::super::locations::work;
use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_policy, SqlFactSite, SqlLocationDemand, StatementPolicySources,
};
use std::sync::Arc;

#[test]
fn retained_roots_require_candidate_syntax_and_share_ast_ownership() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/pruning.sql"
    ));
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/pruning.json"
    )))
    .unwrap();
    work::begin();
    let facts = extract_sql_statement_facts_with_policy(
        sql,
        false,
        None,
        SqlLocationDemand::Full,
        StatementPolicySources {
            collect_grouping: true,
            ..Default::default()
        },
        false,
    );
    let counts = work::finish();
    assert!(!facts.parse_failed);
    assert_eq!(counts.statement_extractions, 1);
    assert_eq!(counts.general_fact_collections, 1);
    let lines = facts
        .grouping_queries
        .iter()
        .map(|query| query.sites.iter().map(|site| site.0).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&lines).unwrap(),
        expected["retainedSelectLines"]
    );
    assert_eq!(counts.grouping_roots, lines.len());
    let discarded = expected["discardedRootLines"].as_array().unwrap();
    assert!(lines.iter().flatten().all(|line| {
        !discarded
            .iter()
            .any(|discarded| discarded.as_u64() == Some(*line as u64))
    }));
    let cloned = facts.clone();
    for (index, (original, cloned)) in facts
        .grouping_queries
        .iter()
        .zip(&cloned.grouping_queries)
        .enumerate()
    {
        assert!(Arc::ptr_eq(&original.query, &cloned.query));
        assert!(Arc::ptr_eq(
            &original.parameter_sites,
            &cloned.parameter_sites
        ));
        for (select, &(line, column)) in original.sites.iter().enumerate() {
            let position = facts
                .variant_locations
                .as_ref()
                .unwrap()
                .position(SqlFactSite::Grouping(index, select))
                .unwrap();
            assert_eq!((position.sql_line, position.sql_column), (line, column));
        }
    }
}
