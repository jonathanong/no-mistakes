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
fn branch_returned_containers_and_callable_cycles_retain_entrypoints() {
    let rows = extract("control-flow-returned-entrypoint-wrappers.ts");
    assert_eq!(rows.len(), 3, "{rows:?}");
    for annotation in ["returns/array", "returns/record", "returns/cycle"] {
        let row = rows
            .iter()
            .find(|row| row.annotation.as_deref() == Some(annotation))
            .unwrap();
        assert!(row.complete && !row.only_value_placeholders, "{row:?}");
        assert!(row.incomplete_reason.is_none(), "{row:?}");
        let statements =
            no_mistakes::codebase::postgres::parse_postgres_sql(row.sql_text.as_ref().unwrap())
                .unwrap();
        // A complete statement remains modifying even when it starts with SELECT.
        assert!(
            matches!(
                statements.as_slice(),
                [
                    sqlparser::ast::Statement::Query(_),
                    sqlparser::ast::Statement::Delete(_)
                ]
            ),
            "{row:?}"
        );
    }
}

#[test]
fn unsupported_returned_shapes_remain_incomplete() {
    let rows = extract("control-flow-returned-entrypoint-unsupported.ts");
    assert_eq!(rows.len(), 3, "{rows:?}");
    let actual = rows
        .iter()
        .map(|row| {
            assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
            assert_eq!(
                row.kind,
                no_mistakes::codebase::postgres::EmbeddedSqlKind::Dynamic,
                "{row:?}"
            );
            (
                row.line,
                row.annotation.as_deref(),
                row.sql_text.as_deref(),
                row.incomplete_reason.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    // Unknown repetition retains zero/one samples, never a complete loop result.
    assert_eq!(
        actual,
        vec![
            (5, None, None, Some("untrusted-array-intrinsics")),
            (
                10,
                Some("returns/unsupported"),
                Some("/* returns/unsupported */ SELECT 7"),
                Some("sql-variants-truncated"),
            ),
            (
                10,
                Some("returns/unsupported"),
                Some("/* returns/unsupported */ SELECT 7; DELETE FROM records"),
                Some("sql-variants-truncated"),
            ),
        ]
    );
}

#[test]
fn discarded_branch_returns_preserve_contextual_callback_invocations() {
    let rows = extract("control-flow-returned-entrypoint-static-branches.ts");
    assert_eq!(rows.len(), 2, "{rows:?}");
    let contextual = rows.iter().find(|row| row.line == 2).unwrap();
    assert!(
        contextual.complete && !contextual.only_value_placeholders,
        "{contextual:?}"
    );
    assert_eq!(contextual.annotation.as_deref(), Some("returns/contextual"));
    assert!(contextual.incomplete_reason.is_none(), "{contextual:?}");
    assert_eq!(
        contextual.kind,
        no_mistakes::codebase::postgres::EmbeddedSqlKind::Composed
    );
    assert_eq!(
        contextual.sql_text.as_deref(),
        Some("/* returns/contextual */ SELECT 8")
    );
    let selected = rows.iter().find(|row| row.line == 9).unwrap();
    assert!(selected.complete, "{selected:?}");
    assert_eq!(
        selected.sql_text.as_deref(),
        Some("/* returns/selected */ SELECT 9; DELETE FROM records")
    );
}
