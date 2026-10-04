use super::{fixture_root, unbounded};

#[test]
fn nested_ctes_restore_parent_relation_visibility() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/nested-cte-scope.sql")).unwrap();
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        vec![("accounts".into(), 2), ("accounts".into(), 4)]
    );
}
