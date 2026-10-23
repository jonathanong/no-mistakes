use super::{
    extract_embedded_sql_from_source,
    tests::{fixture, read_fixture},
    EmbeddedSqlOptions,
};

#[test]
fn undeclared_types_are_diagnostics_and_never_executor_calls() {
    let name = "undeclared-executor-types.ts";
    let options =
        EmbeddedSqlOptions::configured("@example/db", &["query".into(), "execute".into()])
            .with_scoped_executors(&[], &["Executor".into()]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
    assert_eq!(
        facts.calls.len(),
        1,
        "only the declared Executor is evaluated"
    );
    let diagnostics: Vec<_> = facts
        .undeclared_executor_types
        .iter()
        .filter(|call| !call.configured_type)
        .collect();
    assert_eq!(
        super::undeclared::summarize_unresolved_executor_receivers(
            &facts.undeclared_executor_types
        )[0]
        .count,
        1
    );
    assert_eq!(diagnostics.len(), 8);
    assert_eq!(
        diagnostics.iter().map(|call| call.line).collect::<Vec<_>>(),
        [7, 8, 9, 15, 18, 20, 21, 36]
    );
    assert!(diagnostics
        .iter()
        .all(|call| call.type_name == "PoolClient"));
    assert_eq!(diagnostics[4].module, "@example/db/client");
    assert!(diagnostics.iter().all(|call| call.span.0 < call.span.1));
}

#[test]
fn declared_types_and_unscoped_profiles_have_no_undeclared_diagnostics() {
    let name = "undeclared-executor-types.ts";
    for options in [
        EmbeddedSqlOptions::configured("", &["query".into()]),
        EmbeddedSqlOptions::configured("@example/db", &[])
            .with_scoped_executors(&[], &["Executor".into(), "PoolClient".into()]),
        EmbeddedSqlOptions::configured("unrelated", &[]),
    ] {
        let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
        assert!(
            super::summarize_undeclared_executor_types(&facts.undeclared_executor_types).is_empty()
        );
    }
}

#[test]
fn relative_diagnostics_require_the_existing_package_scope_proof() {
    let name = "undeclared-relative-package/query.ts";
    let source = read_fixture(name);
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let initial = extract_embedded_sql_from_source(&fixture(name), &source, &options);
    assert_eq!(initial.undeclared_executor_types.len(), 3);
    let root = fixture("undeclared-relative-package");
    for resolved in [true, false] {
        let mut facts = initial.clone();
        let mut resolutions = 0;
        super::project_relative_scoped_facts(&mut facts, &source, Some(&root), |specifier| {
            resolutions += 1;
            resolved.then(|| root.join(specifier).with_extension("ts"))
        });
        assert_eq!(
            resolutions, 2,
            "unused helper imports need no diagnostic resolution"
        );
        let names = facts
            .undeclared_executor_types
            .iter()
            .map(|call| call.type_name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            if resolved {
                vec!["PoolClient", "DirectClient"]
            } else {
                vec!["DirectClient"]
            }
        );
        assert!(facts
            .undeclared_executor_types
            .iter()
            .all(|call| call.module == "@example/db"));
        assert!(facts.calls.is_empty());
    }
    let mut facts = initial;
    super::project_relative_scoped_facts(&mut facts, &source, None, |_| None);
    assert_eq!(facts.undeclared_executor_types.len(), 1);
    assert_eq!(facts.undeclared_executor_types[0].type_name, "DirectClient");
}

#[test]
fn unused_relative_imports_do_not_request_resolution() {
    let name = "undeclared-relative-package/unused.ts";
    let source = read_fixture(name);
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &source, &options);
    assert!(facts.pending_relative.candidates.is_empty());
    assert!(facts.undeclared_executor_types.is_empty());
}

#[test]
fn scoped_query_diagnostics_match_custom_import_and_static_computed_semantics() {
    let name = "undeclared-methods.ts";
    for (methods, expected) in [(vec!["read".into()], 4), (vec!["execute".into()], 6)] {
        let options = EmbeddedSqlOptions::configured("@example/db", &methods)
            .with_scoped_executors(&[], &["Executor".into()]);
        let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
        assert_eq!(
            facts.calls.len(),
            1,
            "scoped declared .query stays executable"
        );
        assert_eq!(facts.undeclared_executor_types.len(), expected);
    }
}

#[test]
fn summaries_exclude_unresolved_relative_type_owners() {
    let name = "undeclared-relative-package/query.ts";
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
    let summary = super::summarize_undeclared_executor_types(&facts.undeclared_executor_types);
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].type_name, "DirectClient");
    assert_eq!(summary[0].module, "@example/db");
    assert_eq!(summary[0].count, 1);
}

#[test]
fn configured_value_imports_and_their_aliases_are_not_undeclared_types() {
    let name = "undeclared-known-import.ts";
    let options = EmbeddedSqlOptions::configured("@example/db", &["connection".into()]);
    let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
    assert_eq!(facts.calls.len(), 2);
    assert!(facts.undeclared_executor_types.is_empty());
}

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
        [4, 7, 19, 23]
    );
    assert_eq!(
        super::undeclared::summarize_unresolved_executor_receivers(
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

mod review;
