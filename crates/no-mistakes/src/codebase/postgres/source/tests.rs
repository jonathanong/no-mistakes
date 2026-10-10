mod body;
mod conditional_insert_compatibility;
mod conditional_ranges;
mod constraint_spans;
mod ddl;
mod dialect;
mod do_constraints;
mod expression_predicates;
mod expression_roots;
mod expression_spans;
mod indexes;
mod locations;
mod metadata;
mod metadata_conditional;
mod metadata_escape;
mod metadata_escape_values;
mod metadata_qualified_end;
mod metadata_statement_positions;
mod metadata_unicode;
mod parsing;
mod procedural;
mod procedural_empty_statements;
mod procedural_exception;
mod procedural_execute_into;
mod procedural_execute_select;
mod procedural_occurrences;
mod procedural_omit;
mod procedural_quoted_label;
mod schema;
mod schema_virtual;
mod structured_replay;

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
mod insert_recursive;
mod insert_returning;
mod insert_returning_cte;
mod insert_returning_cte_core;
mod insert_returning_quotes;
mod insert_review;

mod wrappers;

mod wrapper_recovery;

mod wrapper_review;

mod wrapper_option_identity;

mod wrapper_metadata;

mod wrapper_boolean_identity;

mod wrapper_with_conflict;

mod wrapper_string_options;

mod wrapper_signed_options;

mod wrapper_atomic_identifiers;
mod wrapper_insert_identifiers;

mod wrapper_repeated_options;

mod wrapper_bare_end_alias;

mod conflict_expressions;

mod conflict_indirection;
