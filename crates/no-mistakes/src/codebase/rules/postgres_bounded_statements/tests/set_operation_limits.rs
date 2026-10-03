use super::{fixture_root, names};

#[test]
fn outer_limits_preserve_non_streaming_set_operation_work() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/set-operation-limit.sql")).unwrap();
    assert_eq!(
        names(&sql),
        ["orders", "orders", "orders", "orders", "orders", "orders"]
    );
}

#[test]
fn wrapped_zero_skips_blocking_inputs_without_trusting_custom_casts() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/wrapped-zero-set-operation.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 24);
    assert_eq!(names(&sql), ["orders"; 7]);
}
