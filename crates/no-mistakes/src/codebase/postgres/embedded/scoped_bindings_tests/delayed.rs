use super::*;

fn facts(name: &str) -> super::super::EmbeddedSqlFileFacts {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("delayed-try-receivers")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("pg", &[])
            .with_scoped_executors(&[], &["PoolClient".into()]),
    )
}

#[test]
fn single_same_function_assignment_proves_the_try_body_receiver() {
    let facts = facts("positive.mts");
    assert_eq!(
        facts
            .calls
            .iter()
            .map(|call| call.sql_text.as_deref())
            .collect::<Vec<_>>(),
        [Some("/* named_try */ SELECT 1"), Some("SELECT 2")]
    );
    assert!(facts.undeclared_executor_types.is_empty());
}

#[test]
fn delayed_receiver_controls_remain_explicit_unresolved_occurrences() {
    let facts = facts("controls.mts");
    assert!(
        facts.calls.is_empty(),
        "unsafe bindings must not become executors"
    );
    assert_eq!(facts.undeclared_executor_types.len(), 8);
    assert!(facts
        .undeclared_executor_types
        .iter()
        .all(|call| call.configured_type
            && call.explicit_annotation
            && call.type_name == "PoolClient"));
}
