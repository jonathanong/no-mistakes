use super::source;
use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions,
};

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
