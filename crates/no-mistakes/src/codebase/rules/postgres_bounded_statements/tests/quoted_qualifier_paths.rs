use super::{fixture_root, unbounded};

#[test]
fn quoted_dots_remain_inside_one_qualifier_component() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/quoted-qualifier-paths.sql")).unwrap();
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        vec![
            ("accounts".into(), 2),
            ("accounts".into(), 6),
            ("accounts".into(), 8)
        ]
    );
}
