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
        [
            Some("/* named_try */ SELECT 1"),
            Some("SELECT 2"),
            Some("/* after_reads */ SELECT 3")
        ]
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
    assert_eq!(facts.undeclared_executor_types.len(), 9);
    assert!(facts
        .undeclared_executor_types
        .iter()
        .all(|call| call.configured_type
            && call.explicit_annotation
            && call.type_name == "PoolClient"));
}

#[test]
fn inner_scope_writes_preserve_the_outer_receiver_and_unproven_types_stay_unknown() {
    let path = super::fixture("delayed-try-scopes.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("pg", &[])
            .with_scoped_executors(&[], &["PoolClient".into()]),
    );
    assert_eq!(
        facts
            .calls
            .iter()
            .filter_map(|call| call.sql_text.as_deref())
            .collect::<Vec<_>>(),
        ["/* retained_outer */ SELECT 1"]
    );
    assert!(facts
        .undeclared_executor_types
        .iter()
        .any(|call| call.type_name == "OtherClient" && !call.configured_type));
    assert!(facts
        .undeclared_executor_types
        .iter()
        .any(|call| call.type_name == "PoolClient" && call.configured_type));
}

#[test]
fn relative_delayed_receivers_require_package_projection_and_keep_both_sql_heads() {
    let path = super::fixture("delayed-try-relative.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("pg", &[])
        .with_scoped_executors(&[], &["PoolClient".into()]);
    for inside in [false, true] {
        let mut facts = extract_embedded_sql_from_source(&path, &source, &options);
        assert!(
            facts.calls.is_empty(),
            "relative ownership is not yet proven"
        );
        assert_eq!(facts.pending_relative.calls.len(), 2);
        let root = path.parent().unwrap();
        super::super::project_relative_scoped_facts(&mut facts, &source, Some(root), |_| {
            Some(if inside {
                root.join("delayed-try-db.mts")
            } else {
                root.parent().unwrap().join("outside-package.mts")
            })
        });
        let sql = facts
            .calls
            .iter()
            .filter_map(|call| call.sql_text.as_deref())
            .collect::<Vec<_>>();
        if inside {
            assert_eq!(sql, ["/* relative_named */ SELECT 1", "SELECT 2"]);
            assert!(facts.undeclared_executor_types.is_empty());
            assert_eq!(facts.matched_type_names, ["PoolClient"]);
        } else {
            assert!(
                sql.is_empty(),
                "an outside package must not prove the receiver"
            );
            assert!(facts.matched_type_names.is_empty());
        }
    }
}

#[test]
fn shorthand_default_and_ts_wrapped_assignment_roots_stay_competing_writes() {
    let path = super::fixture("delayed-target-writes.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("pg", &[])
            .with_scoped_executors(&[], &["PoolClient".into()]),
    );
    assert!(facts.calls.is_empty());
    assert_eq!(facts.undeclared_executor_types.len(), 4);
    assert!(facts
        .undeclared_executor_types
        .iter()
        .all(|call| call.configured_type));
}
