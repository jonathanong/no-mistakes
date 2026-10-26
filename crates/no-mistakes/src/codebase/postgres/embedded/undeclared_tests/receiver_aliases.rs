use super::*;

#[test]
fn explicit_imported_receiver_types_survive_assignment_but_inferred_aliases_do_not() {
    let name = "undeclared-explicit-reassignment.ts";
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&[], &["Executor".into()]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
    assert_eq!(
        facts.calls.len(),
        1,
        "executor proof is unchanged by diagnostic type provenance"
    );
    assert_eq!(
        facts
            .undeclared_executor_types
            .iter()
            .filter(|call| !call.configured_type)
            .map(|call| call.line)
            .collect::<Vec<_>>(),
        [4, 6, 9, 21, 25]
    );
    assert_eq!(
        super::super::undeclared::summarize_unresolved_executor_receivers(
            &facts.undeclared_executor_types
        )[0]
        .count,
        1
    );
    assert!(facts
        .undeclared_executor_types
        .iter()
        .filter(|call| !call.configured_type)
        .all(|call| call.type_name == "PoolClient"));
    assert!(facts
        .undeclared_executor_types
        .iter()
        .filter(|call| !call.configured_type)
        .all(|call| call.known_type));
    let alias = facts
        .undeclared_executor_types
        .iter()
        .find(|call| call.line == 6)
        .expect("ordinary alias diagnostic");
    assert_eq!(alias.receiver_name, "alias");
    assert!(alias.known_type);
    assert!(!alias.explicit_annotation);
}

#[test]
fn constructor_aliases_retain_the_known_imported_type_for_diagnostics() {
    let name = "undeclared-constructor-alias.ts";
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
    assert_eq!(facts.undeclared_executor_types.len(), 1);
    let call = &facts.undeclared_executor_types[0];
    assert_eq!(call.type_name, "PoolClient");
    assert_eq!(call.receiver_name, "connection");
    assert!(call.known_type);
    assert!(
        facts.calls.is_empty(),
        "diagnostic identity is not executor proof"
    );
}
