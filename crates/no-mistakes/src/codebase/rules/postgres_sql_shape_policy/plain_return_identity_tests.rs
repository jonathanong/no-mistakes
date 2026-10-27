use super::super::shape_tests::fixture;
use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

#[test]
fn callable_binding_await_and_lexical_shadow_keep_only_reached_returns() {
    let root = fixture("returned-call-identities");
    for (name, expected) in [
        ("named", 3),
        ("await", 3),
        ("shadow", 5),
        ("recursive", 6),
        ("shadow-edge", 5),
    ] {
        let path = root.join(format!("src/{name}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(
            &path,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        );
        assert_eq!(
            facts
                .fragments
                .iter()
                .map(|fragment| fragment.line)
                .collect::<Vec<_>>(),
            [expected],
            "{name}: {facts:#?}"
        );
    }
}

#[test]
fn unkeyable_member_returns_keep_conservative_recall() {
    let root = fixture("returned-member-owners");
    for name in [
        "class-method",
        "class-field-arrow",
        "this-method",
        "object-method",
        "property-function",
        "named-property-function",
        "unused-method",
        "method-forwarding",
    ] {
        let path = root.join(format!("src/{name}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(
            &path,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        );
        // Member owners are conservative; the unused lexical helpers stay filtered.
        assert_eq!(
            facts
                .fragments
                .iter()
                .map(|fragment| fragment.line)
                .collect::<Vec<_>>(),
            [4],
            "{name}: {facts:#?}",
        );
    }
}
