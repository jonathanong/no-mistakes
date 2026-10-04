use super::{fixture_root, unbounded};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn fixed_catalog_rows_bound_keyed_joins_without_trusting_unknown_calls() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/fixed-catalog-functions.sql")).unwrap();
    assert!(!extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        [3, 4, 5, 6].map(|line| ("accounts".into(), line))
    );
}
