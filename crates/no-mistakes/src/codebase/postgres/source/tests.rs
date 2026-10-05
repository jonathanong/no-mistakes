mod body;
mod ddl;
mod expression_roots;
mod indexes;
mod locations;
mod parsing;
mod procedural;
mod schema;
mod schema_virtual;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source")
            .join(name),
    )
    .unwrap()
}

fn facts(name: &str) -> super::PostgresSqlFacts {
    super::parse_postgres_source(&super::PostgresSqlSource {
        sql: fixture(name),
        file_name: Some(name.into()),
    })
}
