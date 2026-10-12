use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql, EmbeddedSqlOptions, ExecutedPostgresSource, TrustedSqlTag,
};
use std::path::PathBuf;

fn source(name: &str) -> ExecutedPostgresSource {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source")
        .join(name);
    ExecutedPostgresSource {
        file_name: name.to_string(),
        source: std::fs::read_to_string(path).expect("executed source fixture"),
    }
}

#[test]
fn source_api_keeps_execution_sites_annotations_and_complete_alternatives() {
    let sources = [source("reads.ts"), source("writes.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    let statements = extract_executed_postgres_sql(&sources, &options).unwrap();
    assert_eq!(statements.len(), 9);
    assert_eq!(statements[0].file_name, "reads.ts");
    assert_eq!(statements[0].line, 4);
    assert_eq!(statements[0].annotation.as_deref(), Some("posts/by-id"));
    assert!(statements[0]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("sql_placeholder_1"));
    // Interpolation retains its SQL prefix and annotation but is not proven complete.
    assert!(!statements[0].complete);
    assert_eq!(statements[1].annotation.as_deref(), Some("posts/list"));
    assert_eq!(statements[2].file_name, "writes.ts");
    assert!(statements[2]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("UPDATE posts"));
    assert_eq!(statements[3].annotation.as_deref(), Some("posts/one"));
    assert_eq!(statements[4].annotation.as_deref(), Some("posts/two"));
    assert!(statements[3].complete && statements[4].complete);
    assert_eq!(
        statements[3].kind,
        no_mistakes::codebase::postgres::EmbeddedSqlKind::Composed
    );
    assert_eq!(
        statements[4].kind,
        no_mistakes::codebase::postgres::EmbeddedSqlKind::Composed
    );
    assert_eq!(statements[5].annotation, None);
    assert!(!statements[5].complete);
    assert_eq!(statements[6].annotation.as_deref(), Some("posts/runtime"));
    assert!(statements[6]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("sql_placeholder_1"));
    assert!(!statements[6].complete);
    assert_eq!(statements[7].annotation.as_deref(), Some("posts/static"));
    assert!(statements[7].complete);
    assert_eq!(statements[8].sql_text, None);
    assert!(!statements[8].complete);
}

#[test]
fn source_api_rejects_recovered_parse_diagnostics_for_the_whole_batch() {
    let sources = [source("reads.ts"), source("broken.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let error = extract_executed_postgres_sql(&sources, &options).unwrap_err();
    assert!(error.to_string().contains("broken.ts"), "{error}");
}
