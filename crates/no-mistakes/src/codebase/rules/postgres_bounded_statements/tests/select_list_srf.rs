use super::{fixture_root, unbounded};

#[test]
fn data_backed_select_list_srfs_do_not_bound_joined_targets() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/select-list-srf.sql")).unwrap();
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        vec![
            ("accounts".into(), 2),
            ("accounts".into(), 3),
            ("accounts".into(), 4),
            ("accounts".into(), 10),
            ("accounts".into(), 12),
            ("accounts".into(), 14)
        ]
    );
}
