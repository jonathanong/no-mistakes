use super::{fixture_root, unbounded};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn bounded_query_outputs_preserve_nested_pin_read_offenders() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/nested-pin-offenders.sql")).unwrap();
    assert!(!extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        vec![
            ("orders".into(), 2),
            ("orders".into(), 3),
            ("orders".into(), 4)
        ]
    );
}
