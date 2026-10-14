use super::source;
use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlKind, EmbeddedSqlOptions,
    ExecutedPostgresResolvedImport,
};

#[test]
fn source_api_resolves_a_supplied_directory_index_source() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let mut builder = source("imported-builder.ts");
    builder.file_name = "module-dir/index.ts".into();
    let statements = extract_executed_postgres_sql_with_imports(
        &[source("directory-consumer.ts"), builder],
        &options,
        &[],
    )
    .unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "directory-consumer.ts")
        .unwrap();
    assert!(statement.complete, "{statement:?}");
    assert_eq!(statement.annotation.as_deref(), Some("imported/read"));
    assert_eq!(statement.kind, EmbeddedSqlKind::Composed);
}

#[test]
fn source_api_keeps_a_named_import_member_unknown() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(
        &[
            source("named-member-consumer.ts"),
            source("imported-builder.ts"),
        ],
        &options,
        &[],
    )
    .unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "named-member-consumer.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
    assert!(!statement.only_value_placeholders, "{statement:?}");
    assert_eq!(statement.kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn source_api_invalidates_imported_sql_after_a_dynamic_require() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(
        &[
            source("effect-require-dynamic.ts"),
            source("eager-builder.ts"),
        ],
        &options,
        &[],
    )
    .unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "effect-require-dynamic.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
    assert!(!statement.only_value_placeholders, "{statement:?}");
    assert_eq!(statement.kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn source_api_rejects_ambiguous_supplied_identity_and_edges() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let consumer = source("aliased-consumer.ts");
    let builder = source("imported-builder.ts");
    let duplicate = extract_executed_postgres_sql_with_imports(
        &[consumer.clone(), consumer.clone()],
        &options,
        &[],
    )
    .unwrap_err();
    assert!(duplicate.to_string().contains("distinct supplied fileName"));

    let edge = ExecutedPostgresResolvedImport {
        from_file_name: consumer.file_name.clone(),
        specifier: "@example/builder".into(),
        to_file_name: builder.file_name.clone(),
    };
    let invalid = ExecutedPostgresResolvedImport {
        to_file_name: "missing.ts".into(),
        ..edge.clone()
    };
    let unknown = extract_executed_postgres_sql_with_imports(
        &[consumer.clone(), builder.clone()],
        &options,
        &[invalid],
    )
    .unwrap_err();
    assert!(unknown.to_string().contains("supplied source files"));

    let duplicate_edge = extract_executed_postgres_sql_with_imports(
        &[consumer, builder],
        &options,
        &[edge.clone(), edge],
    )
    .unwrap_err();
    assert!(duplicate_edge
        .to_string()
        .contains("duplicate resolvedImports"));
}
