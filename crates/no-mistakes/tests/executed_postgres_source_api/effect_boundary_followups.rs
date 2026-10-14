use super::source;
use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlKind, EmbeddedSqlOptions,
};

#[test]
fn source_api_distinguishes_a_known_imported_append_from_a_suspended_one() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let helper = source("effect-boundary-helper.ts");
    let known = extract_executed_postgres_sql_with_imports(
        &[source("effect-boundary-known.ts"), helper.clone()],
        &options,
        &[],
    )
    .unwrap();
    let known = known
        .iter()
        .find(|statement| statement.file_name == "effect-boundary-known.ts")
        .unwrap();
    assert!(known.complete, "{known:?}");
    assert_eq!(known.kind, EmbeddedSqlKind::Composed);
    assert_eq!(
        known.sql_text.as_deref(),
        Some("/* effects/known */ SELECT id FROM posts WHERE id = 1 ORDER BY id")
    );

    let unknown = extract_executed_postgres_sql_with_imports(
        &[source("effect-boundary-unknown.ts"), helper],
        &options,
        &[],
    )
    .unwrap();
    let unknown = unknown
        .iter()
        .find(|statement| statement.file_name == "effect-boundary-unknown.ts")
        .unwrap();
    assert!(!unknown.complete, "{unknown:?}");
    assert!(!unknown.only_value_placeholders, "{unknown:?}");
    assert_eq!(unknown.kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn source_api_rejects_sql_after_a_nested_computed_await() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(
        &[
            source("effect-boundary-nested-await-consumer.ts"),
            source("effect-boundary-nested-await.ts"),
        ],
        &options,
        &[],
    )
    .unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "effect-boundary-nested-await-consumer.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
    assert!(!statement.only_value_placeholders, "{statement:?}");
    assert_eq!(statement.kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn source_api_does_not_certify_an_indirect_imported_append_argument() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    // The helper receives an indexed expression, so its append effect is not proven.
    let statements = extract_executed_postgres_sql_with_imports(
        &[
            source("effect-boundary-indirect-argument.ts"),
            source("effect-boundary-helper.ts"),
        ],
        &options,
        &[],
    )
    .unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "effect-boundary-indirect-argument.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
    assert!(!statement.only_value_placeholders, "{statement:?}");
    assert_eq!(statement.kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn source_api_keeps_callbacks_exported_through_containers_generic() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    // The exact local call cannot certify an exported callback's unknown caller.
    for fixture in [
        "exported-array-callback.ts",
        "exported-indexed-callback.ts",
        "exported-computed-callback.ts",
        "exported-dynamic-key-callback.ts",
        "exported-sequence-callback.ts",
        "exported-logical-callback.ts",
    ] {
        let statements =
            extract_executed_postgres_sql_with_imports(&[source(fixture)], &options, &[]).unwrap();
        assert_eq!(statements.len(), 1, "{fixture}");
        let statement = &statements[0];
        assert!(!statement.complete, "{fixture}: {statement:?}");
        assert!(
            !statement.only_value_placeholders,
            "{fixture}: {statement:?}"
        );
        assert_eq!(statement.kind, EmbeddedSqlKind::Dynamic, "{fixture}");
    }
}

#[test]
fn source_api_invalidates_imported_sql_after_an_unresolved_require() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let unknown = extract_executed_postgres_sql_with_imports(
        &[
            source("effect-require-unknown.ts"),
            source("eager-builder.ts"),
        ],
        &options,
        &[],
    )
    .unwrap();
    let unknown = unknown
        .iter()
        .find(|statement| statement.file_name == "effect-require-unknown.ts")
        .unwrap();
    assert!(!unknown.complete, "{unknown:?}");
    assert!(!unknown.only_value_placeholders, "{unknown:?}");
    assert_eq!(unknown.kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn source_api_classifies_nested_await_shapes_in_imported_helpers() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(
        &[
            source("suspension-shapes-consumer.ts"),
            source("suspension-shapes.ts"),
        ],
        &options,
        &[],
    )
    .unwrap();
    let statements: Vec<_> = statements
        .iter()
        .filter(|statement| statement.file_name == "suspension-shapes-consumer.ts")
        .collect();
    assert_eq!(statements.len(), 9);
    assert_eq!(
        statements
            .iter()
            .filter(|statement| statement.complete)
            .count(),
        1
    );
    let baseline = statements
        .iter()
        .find(|statement| statement.annotation.as_deref() == Some("suspension/baseline"))
        .unwrap();
    assert!(baseline.complete, "{baseline:?}");
    assert_eq!(
        baseline.sql_text.as_deref(),
        Some("/* suspension/baseline */ SELECT id FROM posts")
    );
    assert_eq!(baseline.kind, EmbeddedSqlKind::Composed);
    for statement in statements {
        if statement.annotation.as_deref() == Some("suspension/baseline") {
            continue;
        }
        assert!(!statement.complete, "{statement:?}");
        assert!(!statement.only_value_placeholders, "{statement:?}");
        assert_eq!(statement.kind, EmbeddedSqlKind::Dynamic);
    }
}
