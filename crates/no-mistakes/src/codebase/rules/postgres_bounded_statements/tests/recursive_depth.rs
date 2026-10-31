use super::{catalog, fixture_root};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn recursive_depth_requires_increasing_capped_steps_bounded_seed_and_key_join() {
    let sql = std::fs::read_to_string(fixture_root().join("column-policy/sql/recursive-depth.sql"))
        .unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 14);
    let catalog = catalog();
    let counts: Vec<_> = facts
        .bounds
        .iter()
        .map(|fact| super::super::evaluate::offenders(fact, &catalog).len())
        .collect();
    assert_eq!(counts, [0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1]);
}
