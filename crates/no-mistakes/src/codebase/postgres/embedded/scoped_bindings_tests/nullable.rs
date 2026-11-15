use super::*;

#[test]
fn nullable_custom_executor_types_keep_each_query_and_reassignment_controls() {
    let root = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors");
    let options = EmbeddedSqlOptions::configured("db-lib", &[])
        .with_scoped_executors(&[], &["PoolClient".into(), "Pool".into()]);
    for name in [
        "direct.ts",
        "subpath.ts",
        "null.ts",
        "pool.ts",
        "pool-subpath.ts",
        "top-direct.ts",
        "top-subpath.ts",
        "top-pool.ts",
        "top-pool-subpath.ts",
    ] {
        let path = root.join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(&path, &source, &options);
        assert_eq!(
            facts.calls.len(),
            if matches!(name, "direct.ts" | "pool.ts") {
                3
            } else {
                2
            },
            "{name}: {facts:?}"
        );
        assert!(facts.undeclared_executor_types.is_empty(), "{name}");
    }
    let path = root.join("controls.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(facts.calls.is_empty());
    assert_eq!(facts.undeclared_executor_types.len(), 5);
}

#[test]
fn known_literal_assignment_sources_cannot_be_overridden_by_type_casts() {
    let path = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors/literal-assignments.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("db-lib", &[])
        .with_scoped_executors(&[], &["PoolClient".into()]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(facts.calls.is_empty(), "{facts:?}");
    assert_eq!(facts.undeclared_executor_types.len(), 12);
}

#[test]
fn configured_callable_function_assignments_remain_executors() {
    let path = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors/callable.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("db-lib", &[])
        .with_scoped_executors(&[], &["Execute".into()]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 4, "{facts:?}");
    assert!(facts.undeclared_executor_types.is_empty());
}

#[test]
fn nullable_optional_query_recovers_pool_client_and_pool_calls() {
    let path = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors/optional-query.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("db-lib", &[])
        .with_scoped_executors(&[], &["PoolClient".into(), "Pool".into()]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    let mut sql = facts
        .calls
        .iter()
        .filter_map(|call| call.sql_text.clone())
        .collect::<Vec<_>>();
    sql.sort();
    assert_eq!(
        sql,
        [
            "UPDATE items SET a=1 WHERE id=$1".to_string(),
            "UPDATE items SET a=2 WHERE id=$1".to_string(),
            "UPDATE items SET a=3 WHERE id=$1 AND account_id=$2".to_string(),
            "UPDATE items SET a=4 WHERE id=$1 AND account_id=$2".to_string(),
        ],
        "{facts:?}"
    );
    assert!(facts.undeclared_executor_types.is_empty(), "{facts:?}");
}
