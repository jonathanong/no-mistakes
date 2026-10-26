use super::super::embedded_call_facts;
use crate::codebase::postgres::{
    extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions,
};

#[test]
fn dynamic_calls_retain_policy_facts_from_complete_prefixes() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-test-state-mutation/fixture/tests/prefix-policy.test.ts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(embedded.calls.len(), 4, "{embedded:#?}");
    for call in &embedded.calls {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic);
        assert!(call.variants.is_empty());
        assert!(call.sql_text.is_some(), "{call:?}");
    }
    let facts = embedded_call_facts(&embedded, false, false);
    assert_eq!(facts.len(), 4);
    assert_eq!(facts[0].statement_kinds[0].kind, "CREATE TABLE");
    assert_eq!(facts[1].setting_uses[0].name, "session_replication_role");
    assert!(facts[2]
        .function_calls
        .iter()
        .any(|call| call.name_parts == ["pg_sleep"]));
    assert!(facts[3]
        .statement_kinds
        .iter()
        .all(|kind| kind.kind != "CREATE TABLE"));
}
