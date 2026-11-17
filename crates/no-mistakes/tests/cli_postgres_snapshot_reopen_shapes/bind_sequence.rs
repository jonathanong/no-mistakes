use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
};

#[test]
fn imported_numeric_constants_share_the_driver_value_bind_sequence() {
    let root = super::case_root("postgres-conflict-ordering", "imported-limit");
    let sources = ["src/root-cause.ts", "src/limits.ts"].map(|relative| {
        let file = root.join(relative);
        ExecutedPostgresSource {
            file_name: file.to_string_lossy().into_owned(),
            source: std::fs::read_to_string(file).unwrap(),
        }
    });
    let statements = extract_executed_postgres_sql_with_imports(
        &sources,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
        &[],
    )
    .unwrap();
    assert_eq!(statements.len(), 2);
    for statement in statements {
        let text = statement.sql_text.as_deref().unwrap();
        assert!(text.contains("LIMIT $4"), "{text}");
        assert!(text.contains("generate_series(1, $5)"), "{text}");
        assert!(text.contains("SELECT $6, i.user_id"), "{text}");
        for index in 1..=6 {
            assert_eq!(text.matches(&format!("${index}")).count(), 1, "{text}");
        }
        assert!(!text.contains("sql_placeholder_"), "{text}");
        assert!(statement.only_value_placeholders);
        assert_eq!(statement.incomplete_reason, None);
    }
}

#[test]
fn scalar_context_projection_never_splices_bound_constant_text() {
    let root = super::scalar_root();
    let sources = ["query.mts", "constants.mts"].map(|relative| {
        let file = root.join(relative);
        ExecutedPostgresSource {
            file_name: file.to_string_lossy().into_owned(),
            source: std::fs::read_to_string(file).unwrap(),
        }
    });
    let statements = extract_executed_postgres_sql_with_imports(
        &sources,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
        &[],
    )
    .unwrap();
    assert_eq!(statements.len(), 5);
    for (statement, expected) in statements.iter().zip([
        Some("SELECT $1, $2 FROM entries OFFSET 1"),
        Some("SELECT CASE WHEN true THEN $1 ELSE $2 END FROM entries OFFSET 1"),
        None,
        Some("SELECT $1, CASE WHEN true THEN $2 ELSE $3 END FROM entries OFFSET 1"),
        Some("SELECT $1, CASE WHEN true THEN $2 ELSE $3 END FROM entries OFFSET 1"),
    ]) {
        assert_eq!(statement.sql_text.as_deref(), expected, "{statement:?}");
        if expected.is_some() {
            assert!(statement.only_value_placeholders, "{statement:?}");
            assert_eq!(statement.incomplete_reason, None);
        } else {
            assert!(!statement.only_value_placeholders);
            assert_eq!(statement.incomplete_reason.as_deref(), Some("opaque-tag-interpolation-effect"));
        }
    }
}
