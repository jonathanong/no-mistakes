use super::{fixture_root, unbounded};

#[test]
fn caller_projected_keys_remain_finite_beside_independent_srfs() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/caller-projection-keys.sql")).unwrap();
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        [8, 9, 10, 12, 13, 16, 17].map(|line| ("accounts".into(), line))
    );
}
