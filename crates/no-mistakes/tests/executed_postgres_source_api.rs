use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql, extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions,
    ExecutedPostgresResolvedImport, ExecutedPostgresSource, TrustedSqlTag,
};
use std::path::{Path, PathBuf};

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
fn source_api_projects_imported_read_from_supplied_sources() {
    let sources = [
        source("imported-consumer.ts"),
        source("imported-builder.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements[0].annotation.as_deref(), Some("imported/read"));
    assert_eq!(
        statements[0].sql_text.as_deref(),
        Some("/* imported/read */ SELECT id FROM posts")
    );
    let legacy = extract_executed_postgres_sql(&sources, &options).unwrap();
    assert!(legacy[0].sql_text.is_none());
}

#[test]
fn source_api_observes_side_effect_only_module_before_imported_read() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    for consumer in ["eager-consumer.ts", "eager-unused-consumer.ts"] {
        let sources = [
            source(consumer),
            source("eager-builder.ts"),
            source("eager-mutator.ts"),
        ];
        let statements =
            extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
        let statement = statements
            .iter()
            .find(|statement| statement.file_name == consumer)
            .unwrap();
        assert!(
            !statement.complete
                || statement
                    .sql_text
                    .as_deref()
                    .unwrap_or("")
                    .contains("UPDATE posts"),
            "{statement:?}"
        );
    }
}

#[test]
fn source_api_resolves_explicit_supplied_alias_edge() {
    let sources = [source("aliased-consumer.ts"), source("imported-builder.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let edges = [ExecutedPostgresResolvedImport {
        from_file_name: "aliased-consumer.ts".into(),
        specifier: "@example/builder".into(),
        to_file_name: "imported-builder.ts".into(),
    }];
    let statements =
        extract_executed_postgres_sql_with_imports(&sources, &options, &edges).unwrap();
    assert_eq!(statements[0].annotation.as_deref(), Some("imported/read"));
}

#[test]
fn source_api_resolves_unambiguous_extensionless_supplied_import() {
    let sources = [
        source("extensionless-consumer.ts"),
        source("imported-builder.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements[0].annotation.as_deref(), Some("imported/read"));
}

#[test]
fn source_api_forwards_exact_object_callbacks_across_imports() {
    let sources = [source("ordered-consumer.ts"), source("ordered-helper.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].annotation.as_deref(), Some("ordered/list"));
    assert!(statements[0].complete);
}

#[test]
fn source_api_never_promotes_spreads_getters_or_unknown_write_alternatives_to_a_read() {
    let sources = [source("ordered-unsafe.ts"), source("ordered-helper.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 3);
    assert!(statements
        .iter()
        .all(|statement| !statement.complete && statement.sql_text.is_none()));
}

#[test]
fn source_api_does_not_promote_an_opaque_imported_append() {
    let sources = [source("opaque-consumer.ts"), source("opaque-append.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(!statements[0]
        .sql_text
        .as_deref()
        .unwrap_or("")
        .contains("SELECT id FROM eligible"));
}

#[test]
fn source_api_preserves_the_full_with_body_for_callers_to_classify() {
    let sources = [source("with-branches.ts"), source("with-builder.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 2);
    assert!(statements.iter().all(|statement| statement.complete));
    assert!(statements[0]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("WITH source AS (SELECT"));
    assert!(statements[1]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("WITH changed AS (DELETE"));
    assert_ne!(statements[0].annotation, statements[1].annotation);
}

#[test]
fn source_api_projects_imported_mutating_builder_and_nested_count() {
    let sources = [
        source("search-consumer.ts"),
        source("search-builder.ts"),
        source("search-append.ts"),
        source("search-count.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["beginTransaction".into()], &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 3);
    assert_eq!(statements[0].annotation.as_deref(), Some("search/list"));
    assert_eq!(statements[1].annotation.as_deref(), Some("search/count"));
    assert_eq!(statements[2].annotation.as_deref(), Some("search/list"));
    assert!(statements.iter().all(|statement| statement.complete));
    assert!(statements[0]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("WITH eligible AS"));
    assert!(statements[1]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("SELECT COUNT(*)"));
}

#[test]
fn source_api_rejects_recovered_parse_diagnostics_for_the_whole_batch() {
    let sources = [source("reads.ts"), source("broken.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let error = extract_executed_postgres_sql(&sources, &options).unwrap_err();
    assert!(error.to_string().contains("broken.ts"), "{error}");
}

#[test]
fn source_api_reports_the_first_malformed_source_in_input_order() {
    let mut first = source("broken.ts");
    first.file_name = "first.ts".into();
    let mut second = source("broken.ts");
    second.file_name = "second.ts".into();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let error = extract_executed_postgres_sql(&[first, second], &options).unwrap_err();
    assert!(error.to_string().contains("first.ts"), "{error}");
}

#[test]
fn source_api_ignores_a_worker_inherited_cache_for_the_same_file_name() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        no_mistakes::ast::with_request_parse_cache(|| {
            let cached = source("reads.ts");
            no_mistakes::ast::with_program(Path::new("reads.ts"), &cached.source, |_, _| ())
                .unwrap();
            let mut supplied = source("writes.ts");
            supplied.file_name = "reads.ts".into();
            let options = EmbeddedSqlOptions::configured("@example/db", &[]);
            let statements = extract_executed_postgres_sql(&[supplied], &options).unwrap();
            assert_eq!(statements[0].annotation.as_deref(), Some("posts/update"));
        });
    });
}
