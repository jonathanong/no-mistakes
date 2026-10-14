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

#[path = "executed_postgres_source_api/review_followups.rs"]
mod review_followups;

#[path = "executed_postgres_source_api/effect_boundary_followups.rs"]
mod effect_boundary_followups;

#[path = "executed_postgres_source_api/import_boundary_followups.rs"]
mod import_boundary_followups;

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
fn source_api_never_unwraps_an_unawaited_sql_promise() {
    let sources = [source("async-consumer.ts"), source("async-builder.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 2);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
    assert_eq!(
        statements[1].sql_text.as_deref(),
        Some("/* async/read */ SELECT id FROM posts")
    );
    assert!(statements[1].complete);
}

#[test]
fn source_api_never_certifies_sql_across_imported_async_suspension() {
    let sources = [
        source("async-suspended-consumer.ts"),
        source("async-suspended-builder.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 3);
    for statement in statements {
        assert!(!statement.complete, "{statement:?}");
        assert!(statement.sql_text.is_none(), "{statement:?}");
    }
}

#[test]
fn source_api_discards_executor_events_recorded_after_async_suspension() {
    let sources = [source("async-event-consumer.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].file_name, "async-event-consumer.ts");
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
}

#[test]
fn source_api_retains_the_generic_call_of_an_exported_helper() {
    let sources = [source("exported-generic.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
}

#[test]
fn source_api_retains_generic_callbacks_exposed_by_exported_records() {
    let sources = [source("exported-record.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
}

#[test]
fn source_api_retains_generic_inline_callbacks_in_exported_records() {
    let sources = [source("exported-inline-record.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
}

#[test]
fn source_api_retains_generic_callbacks_exposed_by_conditional_exports() {
    let sources = [source("exported-conditional.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
}

#[test]
fn source_api_keeps_exported_object_spread_callbacks_generic() {
    let sources = [source("exported-spread.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
}

#[test]
fn source_api_invalidates_a_builder_after_an_ordinary_getter_read() {
    let sources = [source("member-getter.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].sql_text.is_none(), "{:?}", statements[0]);
}

#[test]
fn source_api_preserves_proven_intrinsic_string_raw_text() {
    let sources = [source("intrinsic-raw.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert_eq!(
        statements[0].sql_text.as_deref(),
        Some("/* raw/read */ SELECT id FROM posts")
    );
    assert!(statements[0].complete);
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
        assert!(!statement.complete, "{statement:?}");
        assert!(!statement.only_value_placeholders, "{statement:?}");
    }
}

#[test]
fn source_api_fails_closed_when_an_eager_relative_module_is_not_supplied() {
    let sources = [source("eager-consumer.ts"), source("eager-builder.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "eager-consumer.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
}

#[test]
fn source_api_does_not_record_sql_before_a_later_executor_argument_getter() {
    let sources = [source("executor-getter.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
}

#[test]
fn source_api_prefers_the_canonical_typescript_source_candidate() {
    let sources = [
        source("priority-consumer.ts"),
        source("priority-helper.ts"),
        source("priority-helper.tsx"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "priority-consumer.ts")
        .unwrap();
    assert!(statement.complete, "{statement:?}");
    assert_eq!(
        statement.sql_text.as_deref(),
        Some("/* preferred source */ SELECT 1")
    );
}

#[test]
fn source_api_resolves_a_static_namespace_member_from_supplied_source() {
    let sources = [
        source("namespace-consumer.ts"),
        source("priority-helper.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "namespace-consumer.ts")
        .unwrap();
    assert!(statement.complete, "{statement:?}");
    assert_eq!(
        statement.sql_text.as_deref(),
        Some("/* preferred source */ SELECT 1")
    );
}

#[test]
fn source_api_observes_eager_commonjs_dependency_before_imported_read() {
    let sources = [
        source("cjs-eager-consumer.ts"),
        source("eager-builder.ts"),
        source("eager-mutator.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "cjs-eager-consumer.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
}

#[test]
fn source_api_keeps_read_before_later_commonjs_mutator() {
    let sources = [
        source("cjs-late-consumer.ts"),
        source("eager-builder.ts"),
        source("eager-mutator.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "cjs-late-consumer.ts")
        .unwrap();
    assert!(statement.complete, "{statement:?}");
    assert_eq!(
        statement.sql_text.as_deref(),
        Some("/* eager import */ SELECT 1")
    );
}

#[test]
fn source_api_initializes_wrapped_commonjs_require_before_later_read() {
    let sources = [
        source("cjs-wrapped-consumer.ts"),
        source("eager-builder.ts"),
        source("eager-mutator.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "cjs-wrapped-consumer.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
    assert!(statement.sql_text.is_none(), "{statement:?}");
}

#[test]
fn source_api_initializes_awaited_dynamic_import_before_later_read() {
    let sources = [
        source("dynamic-import-consumer.ts"),
        source("eager-builder.ts"),
        source("eager-mutator.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "dynamic-import-consumer.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
    assert!(statement.sql_text.is_none(), "{statement:?}");

    let legacy = extract_executed_postgres_sql(&sources, &options).unwrap();
    let legacy_statement = legacy
        .iter()
        .find(|statement| statement.file_name == "dynamic-import-consumer.ts")
        .unwrap();
    assert!(!legacy_statement.complete, "{legacy_statement:?}");
    assert!(legacy_statement.sql_text.is_none(), "{legacy_statement:?}");
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
