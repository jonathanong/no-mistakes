mod body;
mod conditional_ranges;
mod ddl;
mod do_constraints;
mod expression_roots;
mod indexes;
mod locations;
mod metadata;
mod metadata_conditional;
mod metadata_escape;
mod metadata_escape_values;
mod metadata_unicode;
mod parsing;
mod procedural;
mod schema;
mod schema_virtual;

pub(super) fn fixture(name: &str) -> String {
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

mod query;

mod insert;
mod insert_review;

mod wrappers;

mod wrapper_recovery;
