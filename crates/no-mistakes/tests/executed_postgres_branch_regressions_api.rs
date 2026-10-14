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
    let divergent = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("shapes/divergent"))
        .collect::<Vec<_>>();
    assert!(!divergent.is_empty(), "{rows:?}");
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
fn branch_returned_private_functions_are_evaluated_as_public_entrypoints() {
    let rows = extract("control-flow-branch-returned-functions.ts");
    for annotation in ["returns/left", "returns/right"] {
        let variants = rows
            .iter()
            .filter(|row| row.annotation.as_deref() == Some(annotation))
            .collect::<Vec<_>>();
        assert!(!variants.is_empty(), "{rows:?}");
        // Lexical extraction alone sees only the pre-append SELECT template.
        assert!(
            variants.iter().all(|row| row.complete
                && row
                    .sql_text
                    .as_ref()
                    .unwrap()
                    .contains("DELETE FROM records")),
            "{variants:?}"
        );
    }
}
