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
