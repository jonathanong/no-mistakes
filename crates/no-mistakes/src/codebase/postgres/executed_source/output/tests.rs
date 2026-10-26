use crate::codebase::postgres::{
    extract_executed_postgres_sql, extract_executed_postgres_sql_with_imports, EmbeddedSqlKind,
    EmbeddedSqlOptions, ExecutedPostgresSource,
};

fn extract(name: &str) -> Vec<super::ExecutedPostgresSql> {
    extract_with_imports(name, false)
}

fn extract_with_imports(name: &str, resolve_imports: bool) -> Vec<super::ExecutedPostgresSql> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    let sources = [ExecutedPostgresSource {
        file_name: name.into(),
        source: std::fs::read_to_string(path).unwrap(),
    }];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    if resolve_imports {
        extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap()
    } else {
        extract_executed_postgres_sql(&sources, &options).unwrap()
    }
}

#[test]
fn import_projection_preserves_every_retained_variant_and_its_incompleteness() {
    let direct = extract("variants-append-breadth.ts");
    let resolved = extract_with_imports("variants-append-breadth.ts", true);
    assert_eq!(resolved[..37], direct[..37]);
}

#[test]
fn supplied_sql_reports_retained_variants_as_incomplete_when_the_site_is_truncated() {
    let statements = extract("variants-append-breadth.ts");
    assert_eq!(statements.len(), 39);
    // Every explored alternative is concrete, but none proves the site exhaustive.
    for statement in &statements[..37] {
        assert!(statement.sql_text.is_some());
        assert!(!statement.complete);
        assert!(!statement.only_value_placeholders);
        assert_eq!(statement.kind, EmbeddedSqlKind::Dynamic);
        assert_eq!(
            statement.incomplete_reason.as_deref(),
            Some("sql-variants-truncated")
        );
    }
    for statement in &statements[37..] {
        assert!(statement.complete);
        assert_eq!(statement.kind, EmbeddedSqlKind::Composed);
        assert_eq!(statement.incomplete_reason, None);
    }
}

#[test]
fn supplied_sql_does_not_claim_loop_exploration_is_exhaustive() {
    let statements = extract("variants-append-loops.ts");
    assert_eq!(statements.len(), 11);
    assert!(statements[..10].iter().all(|statement| !statement.complete
        && !statement.only_value_placeholders
        && statement.kind == EmbeddedSqlKind::Dynamic
        && statement.incomplete_reason.as_deref() == Some("sql-variants-truncated")));
    let unchanged = &statements[10];
    assert_eq!(unchanged.sql_text.as_deref(), Some("SELECT 6"));
    assert!(unchanged.complete);
    assert!(!unchanged.only_value_placeholders);
    assert_eq!(unchanged.kind, EmbeddedSqlKind::ImmutableLocal);
    assert_eq!(unchanged.incomplete_reason, None);
}
