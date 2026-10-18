use super::source;
use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, TrustedSqlTag,
};

#[test]
fn source_api_judges_same_file_sql_tag_builder() {
    // Non-exported same-file builder returning a configured `sql` tag.
    // `${id}` is a value bind: the statement is judged and has no failure reason.
    let sources = [source("same-file-sql-builder.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1, "{statements:?}");
    assert_eq!(
        statements[0].sql_text.as_deref(),
        Some("/* orders/by-id */ SELECT id FROM orders WHERE account_id = sql_placeholder_1")
    );
    // A trusted-tag value bind is judged: the structure is known and there is
    // no failure reason. `complete` stays false because the bind is not authored SQL.
    assert!(!statements[0].complete, "{:?}", statements[0]);
    assert!(statements[0].only_value_placeholders, "{:?}", statements[0]);
    assert_eq!(statements[0].incomplete_reason, None, "{:?}", statements[0]);
}

#[test]
fn source_api_does_not_execute_an_unawaited_dynamic_module_before_the_read() {
    let sources = [
        source("dynamic-unawaited-consumer.ts"),
        source("eager-builder.ts"),
        source("eager-mutator.ts"),
    ];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    let statement = statements
        .iter()
        .find(|statement| statement.file_name == "dynamic-unawaited-consumer.ts")
        .unwrap();
    assert!(!statement.complete, "{statement:?}");
    assert!(statement.sql_text.is_none(), "{statement:?}");
}

#[test]
fn source_api_refreshes_the_selected_method_receiver_after_argument_effects() {
    let sources = [source("receiver-argument.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(
        !statements[0].complete
            || statements[0]
                .sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains("UPDATE posts")),
        "{:?}",
        statements[0]
    );
}

#[test]
fn source_api_does_not_retain_sql_across_an_opaque_call_initializer() {
    let sources = [source("opaque-initializer.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert!(!statements[0].complete, "{:?}", statements[0]);
}

#[test]
fn source_api_limits_variants_merged_from_independent_entrypoints() {
    let sources = [source("merged-entrypoint-variant-limit.ts")];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let statements = extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
    assert_eq!(statements.len(), 1);
    assert_eq!(
        statements[0].incomplete_reason.as_deref(),
        Some("variant-limit"),
        "{:?}",
        statements[0]
    );
    assert!(!statements[0].complete);
    assert!(statements[0].sql_text.is_none());
}
