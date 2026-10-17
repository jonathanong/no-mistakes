use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
};
use std::path::PathBuf;

fn assert_frozen_output(relative: &str, names: &[&str]) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    let sources = names
        .iter()
        .map(|name| ExecutedPostgresSource {
            file_name: (*name).into(),
            source: std::fs::read_to_string(directory.join(name)).unwrap(),
        })
        .collect::<Vec<_>>();
    let rows = extract_executed_postgres_sql_with_imports(
        &sources,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
        &[],
    )
    .unwrap();
    // Frozen from the exact merged-main 8cb93487 public supplied-source API.
    // Exact complete SQL and explicit failure reasons prevent Unknown parity
    // from disguising a lost module mutation or same-owner lexical capture.
    let expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(directory.join("expected-executed-sql.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(serde_json::to_value(rows).unwrap(), expected);
}

#[test]
fn module_slot_writes_survive_branches_and_keep_local_closures_distinct() {
    assert_frozen_output(
        "../../fixtures/postgres-facts/executed-source/module-capture-slots",
        &["entry.mts", "shared.mts"],
    );
}

#[test]
fn branch_returned_and_dynamic_module_functions_preserve_public_output() {
    assert_frozen_output(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/module-captures",
        &["entry.mts", "lazy-mutator.mts", "pure.mts", "shared.mts"],
    );
}
