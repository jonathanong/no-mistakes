use super::{extract_embedded_sql_from_source, fixture, EmbeddedSqlOptions};

#[test]
fn configured_commonjs_namespace_and_query_aliases_require_positive_proof() {
    let path = fixture("commonjs-positive.cjs");
    let source = std::fs::read_to_string(&path).expect("saved CommonJS fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(
        facts.calls.iter().map(|call| call.line).collect::<Vec<_>>(),
        [2, 4, 6, 7]
    );
    assert_eq!(
        facts
            .calls
            .iter()
            .map(|call| call.sql_text.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("SELECT 1"),
            Some("SELECT 2"),
            Some("SELECT 3"),
            Some("SELECT 4")
        ]
    );
    // A namespace `.query` needs the configured `query` export selector.
    let without_query = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    assert!(
        extract_embedded_sql_from_source(&path, &source, &without_query)
            .calls
            .is_empty()
    );
}

#[test]
fn namespace_mutation_revokes_member_proof_but_not_captured_query_function() {
    let path = fixture("commonjs-provenance.cjs");
    let source = std::fs::read_to_string(&path).expect("saved CommonJS mutation fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 3"));
}

#[test]
fn later_declared_mutators_revoke_earlier_member_sites() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["beginTransaction".into()], &[]);
    let namespace = fixture("namespace-late-mutator.cjs");
    let source = std::fs::read_to_string(&namespace).unwrap();
    assert!(
        extract_embedded_sql_from_source(&namespace, &source, &options)
            .calls
            .is_empty()
    );
    let factory = fixture("factory-late-mutator.ts");
    let source = std::fs::read_to_string(&factory).unwrap();
    let facts = extract_embedded_sql_from_source(&factory, &source, &options);
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT direct late")
    );
}

#[test]
fn commonjs_namespace_shadow_does_not_inherit_imported_callable_name() {
    let path = fixture("commonjs-import-shadow.ts");
    let source = std::fs::read_to_string(&path).expect("saved CommonJS shadow fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(facts.calls[0].line, 7);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 2"));
}

#[test]
fn commonjs_cache_invalidation_revokes_configured_subpaths() {
    let path = fixture("commonjs-subpath-cache.cjs");
    let source = std::fs::read_to_string(&path).expect("saved CommonJS subpath fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(facts.calls.is_empty());
}

#[test]
fn unmutated_commonjs_subpath_remains_proven() {
    let path = fixture("commonjs-subpath-positive.cjs");
    let source = std::fs::read_to_string(&path).expect("saved CommonJS subpath fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(facts.calls[0].line, 3);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 11"));
}

#[test]
fn empty_options_do_not_promote_namespace_destructuring() {
    let path = fixture("unconfigured-namespace-query.ts");
    let source = std::fs::read_to_string(&path).expect("saved unconfigured namespace fixture");
    let facts = extract_embedded_sql_from_source(&path, &source, &EmbeddedSqlOptions::default());
    assert!(facts.calls.is_empty());
}

#[test]
fn unrelated_member_write_preserves_factory_executor_call() {
    let path = fixture("factory-unrelated-member-write.ts");
    let source = std::fs::read_to_string(&path).expect("saved factory member write fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["beginTransaction".into()], &[]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(facts.calls[0].line, 6);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 12"));
}

#[test]
fn factory_member_mutations_keep_only_callable_executor_proof() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["beginTransaction".into()], &[]);
    for (name, sql) in [
        (
            "factory-member-query-assignment.ts",
            "SELECT direct assignment",
        ),
        ("factory-member-query-delete.ts", "SELECT direct delete"),
        ("factory-member-query-computed.ts", "SELECT direct computed"),
        ("factory-member-query-escape.ts", "SELECT direct escape"),
    ] {
        let path = fixture(name);
        let source = std::fs::read_to_string(&path).expect("saved factory mutation fixture");
        let facts = extract_embedded_sql_from_source(&path, &source, &options);
        assert_eq!(facts.calls.len(), 1, "{name}: {:#?}", facts.calls);
        assert_eq!(facts.calls[0].sql_text.as_deref(), Some(sql), "{name}");
    }
}
