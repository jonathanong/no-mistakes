use super::{fixture_root, unbounded};

#[test]
fn unknown_projection_calls_do_not_prove_scalar_cardinality() {
    let root = fixture_root();
    let sql = std::fs::read_to_string(root.join("sql/unknown-projection-cardinality.sql")).unwrap();
    let definition = std::fs::read_to_string(root.join("sql/custom-srf-definition.sql")).unwrap();
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(&definition).parse_failed);
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        vec![
            ("accounts".into(), 2),
            ("accounts".into(), 4),
            ("accounts".into(), 6),
            ("accounts".into(), 8),
            ("accounts".into(), 10),
            ("accounts".into(), 21),
            ("accounts".into(), 24)
        ]
    );
}
