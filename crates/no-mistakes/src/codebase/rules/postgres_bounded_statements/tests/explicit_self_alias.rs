use super::{fixture_root, names};
use crate::codebase::postgres::{extract_sql_statement_facts, SqlBoundItemKind};

#[test]
fn an_explicit_self_alias_does_not_capture_a_qualified_outer_relation() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/explicit-self-alias-lateral.sql"))
        .unwrap();
    // The nested lateral query skips the explicit `accounts` alias and reads the outer row.
    // Only that unbounded outer scan is reported; the middle scan is pinned by unique email.
    assert_eq!(names(&sql), ["accounts"]);

    let facts = extract_sql_statement_facts(&sql);
    let outer = &facts.bounds[0].query.items;
    assert!(!outer[0].alias_explicit);
    let SqlBoundItemKind::Query(middle) = &outer[1].kind else {
        panic!("expected the lateral middle query");
    };
    assert!(middle.items[0].alias_explicit);
    assert!(!middle.items[1].lateral);
}
