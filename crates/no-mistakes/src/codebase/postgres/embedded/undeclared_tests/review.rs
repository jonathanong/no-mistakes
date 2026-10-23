use super::*;

#[test]
fn undeclared_receivers_respect_type_shadows_and_hoisted_binding_frames() {
    let name = "undeclared-lexical-scopes.ts";
    let source = read_fixture(name);
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &source, &options);
    let expected: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// diagnostic:").then_some(index as u32 + 1))
        .collect();
    assert_eq!(
        facts
            .undeclared_executor_types
            .iter()
            .map(|call| call.line)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(facts.undeclared_executor_types[0].type_name, "OtherClient");
    assert!(
        facts.calls.is_empty(),
        "diagnostics never establish executor proof"
    );
}

#[test]
fn undeclared_member_receivers_preserve_imported_type_and_lexical_ownership() {
    let name = "undeclared-member-receivers.ts";
    let source = read_fixture(name);
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&[], &["Executor".into()]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &source, &options);
    let expected: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// diagnostic").then_some(index as u32 + 1))
        .collect();
    assert_eq!(
        facts
            .undeclared_executor_types
            .iter()
            .map(|call| call.line)
            .collect::<Vec<_>>(),
        expected
    );
    assert!(facts
        .undeclared_executor_types
        .iter()
        .all(|call| call.type_name == "PoolClient"));
    assert_eq!(
        facts.calls.len(),
        1,
        "only the declared bare receiver is evaluated"
    );
    let declared = options.with_scoped_executors(&[], &["PoolClient".into(), "Executor".into()]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &source, &declared);
    assert_eq!(facts.undeclared_executor_types.len(), expected.len());
    assert!(facts
        .undeclared_executor_types
        .iter()
        .all(|call| call.configured_type));
    assert!(
        super::super::summarize_undeclared_executor_types(&facts.undeclared_executor_types)
            .is_empty()
    );
    assert_eq!(
        super::super::undeclared::summarize_unresolved_executor_receivers(
            &facts.undeclared_executor_types
        )[0]
        .count,
        expected.len()
    );
    assert_eq!(
        facts.calls.len(),
        1,
        "configured type names do not prove complex receivers"
    );
}

#[test]
fn unsupported_receiver_boundaries_do_not_invent_imported_type_provenance() {
    let name = "undeclared-member-boundaries.ts";
    let source = read_fixture(name);
    let facts = extract_embedded_sql_from_source(
        &fixture(name),
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    let expected = source
        .lines()
        .position(|line| line.contains("// diagnostic"))
        .unwrap() as u32
        + 1;
    assert_eq!(
        facts
            .undeclared_executor_types
            .iter()
            .map(|call| call.line)
            .collect::<Vec<_>>(),
        [expected]
    );
    assert!(facts.calls.is_empty());
}
