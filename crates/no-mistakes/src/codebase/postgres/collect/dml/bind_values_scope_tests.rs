use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn stable_member_escape_does_not_poison_another_scopes_legacy_sql_splice() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("../issue-3151-real/fixture");
    let path = root.join("legacy-cross-scope.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    let embedded = extract_embedded_sql_from_source(&path, &source, &options);
    let [call] = embedded.calls.as_slice() else {
        panic!("one saved legacy executor call");
    };
    assert_eq!(
        call.sql_text.as_deref(),
        Some("SELECT role FROM group_members")
    );
    assert!(call.recovered_placeholder_positions.is_empty());
    assert!(call.bound_value_origins.is_empty());
    assert!(call.bound_value_sql.is_none());
}
