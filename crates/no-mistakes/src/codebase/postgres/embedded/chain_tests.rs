use super::{EmbeddedSqlOptions, extract_embedded_sql_from_source};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn extract(name: &str) -> super::EmbeddedSqlFileFacts {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture");
    extract_embedded_sql_from_source(
        &fixture(name),
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
}

#[path = "chain_tests_cases/part1.rs"]
mod part1;

#[path = "chain_tests_cases/part2.rs"]
mod part2;
