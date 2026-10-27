use super::fixture_root;
use crate::codebase::postgres::{extract_sql_statement_facts, SqlBoundInputMode, SqlBoundItemKind};

#[test]
fn outer_caps_preserve_only_blocking_input_reads() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/blocking-query-wrappers.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert!(facts.bounds[0].query.capped);
    assert_eq!(
        facts.bounds[0].query.input_mode,
        SqlBoundInputMode::Streaming
    );
    let SqlBoundItemKind::Query(inner) = &facts.bounds[0].query.items[0].kind else {
        panic!("expected derived query");
    };
    assert_eq!(inner.input_mode, SqlBoundInputMode::Blocking);
    assert_eq!(facts.bounds[7].query.input_mode, SqlBoundInputMode::Skipped);
}

#[test]
fn compacted_wrappers_keep_blocking_reads_but_zero_executes_no_inputs() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/compacted-blocking-wrappers.sql"))
        .unwrap();
    assert!(!extract_sql_statement_facts(&sql).parse_failed);
}
