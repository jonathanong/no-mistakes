use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
    ExecutedPostgresSql, TrustedSqlTag,
};
use std::path::PathBuf;

fn extract(name: &str) -> Vec<ExecutedPostgresSql> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source")
        .join(name);
    extract_executed_postgres_sql_with_imports(
        &[ExecutedPostgresSource {
            file_name: name.into(),
            source: std::fs::read_to_string(path).unwrap(),
        }],
        &EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
            TrustedSqlTag {
                module: "@example/db".into(),
                name: "sql".into(),
            },
        ]),
        &[],
    )
    .unwrap()
}

#[test]
fn divergent_dense_shapes_do_not_fabricate_a_known_length() {
    let rows = extract("control-flow-branch-shapes.ts");
    let divergent = rows.iter().filter(|row| row.line == 15).collect::<Vec<_>>();
    assert_eq!(divergent.len(), 1, "{rows:?}");
    // A shape failure must block both branches rather than prove the SELECT arm.
    for row in divergent {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert_eq!(
            row.incomplete_reason.as_deref(),
            Some("conditional-collection-shape"),
            "{row:?}"
        );
    }
    let dense = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("shapes/dense"))
        .collect::<Vec<_>>();
    assert_eq!(dense.len(), 1, "{rows:?}");
    assert!(dense[0].complete, "{rows:?}");
    assert_eq!(
        dense[0].sql_text.as_deref(),
        Some("/* shapes/dense */ SELECT 2")
    );
}

#[test]
fn branch_returned_private_functions_remain_safely_incomplete() {
    let rows = extract("control-flow-branch-returned-functions.ts");
    assert_eq!(rows.len(), 2, "{rows:?}");
    // Follow-up #1973 will recover these returned entrypoints. Until then the
    // lexical pre-append prefix must never be certified as the full statement.
    for row in rows {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert_eq!(
            row.incomplete_reason.as_deref(),
            Some("unresolved-sql-fragment")
        );
    }
}

#[test]
fn conditional_shapes_cannot_skip_later_callback_mutations() {
    let rows = extract("control-flow-conditional-callback-shapes.ts");
    assert_eq!(rows.len(), 10, "{rows:?}");
    for row in rows {
        if matches!(row.line, 35 | 44) {
            assert!(row.complete, "{row:?}");
            assert_eq!(
                row.sql_text.as_deref(),
                Some("SELECT 1; DELETE FROM records")
            );
        } else if row.line == 51 {
            // Both push-length alternatives were already recovered before the fix.
            assert!(row.complete, "{row:?}");
        } else {
            assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        }
    }
}

#[test]
fn operand_failure_cannot_discard_accessor_effects() {
    let rows = extract("control-flow-operand-failure-effects.ts");
    assert_eq!(rows.len(), 7, "{rows:?}");
    assert_eq!(
        rows.iter().filter(|row| row.complete).count(),
        2,
        "{rows:?}"
    );
    for row in rows {
        let pure = matches!(row.line, 26 | 32);
        assert_eq!(row.complete, pure, "{row:?}");
        assert!(!row.only_value_placeholders, "{row:?}");
    }
}

#[test]
fn builder_coercion_observes_later_supported_operand_effects() {
    let rows = extract("control-flow-late-builder-coercion.ts");
    assert_eq!(rows.len(), 5, "{rows:?}");
    for row in &rows {
        if matches!(row.line, 17 | 29) {
            // Builder binary coercion was already conservative before this fix.
            assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
            assert_eq!(
                row.incomplete_reason.as_deref(),
                Some("opaque-binary-coercion")
            );
            continue;
        }
        assert!(row.complete, "{rows:?}");
        let text = if matches!(row.line, 23 | 29) {
            "SELECT 1"
        } else {
            "SELECT 1; DELETE FROM records"
        };
        assert_eq!(row.sql_text.as_deref(), Some(text), "{rows:?}");
    }
}
