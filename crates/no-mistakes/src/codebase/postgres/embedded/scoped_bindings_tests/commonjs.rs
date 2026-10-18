use super::{extract_embedded_sql_from_source, fixture, EmbeddedSqlOptions};

#[test]
fn configured_commonjs_namespace_and_query_aliases_require_positive_proof() {
    let path = fixture("commonjs-provenance.cjs");
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
