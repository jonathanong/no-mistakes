use super::{fixture_root, unbounded};

#[test]
fn set_operation_arms_do_not_share_relation_aliases() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/set-arm-alias-scope.sql")).unwrap();
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        vec![
            ("accounts".into(), 2),
            ("accounts".into(), 4),
            ("accounts".into(), 8),
            ("accounts".into(), 10)
        ]
    );
}
