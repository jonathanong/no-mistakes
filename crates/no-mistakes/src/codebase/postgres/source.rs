//! Standalone source facts: one prepared token inventory, one AST per statement.

mod adjacent_strings;
mod alter;
mod body;
mod columns;
mod completeness;
mod conditional;
mod conditional_source;
mod ddl;
mod diagnostics;
mod drop_facts;
mod execute;
mod execute_preparation;
mod expression_roots;
mod expressions;
mod generated;
mod index_only;
mod indexes;
mod insert;
mod locations;
mod metadata;
mod metadata_preparation;
mod parsing;
mod procedural;
mod procedural_occurrences;
mod projection;
mod query;
mod recovery;
mod type_facts;
mod types;
mod wrappers;
pub use types::*;

/// Parse source text without a repository scan, catalog, or database connection.
pub fn parse_postgres_source(source: &PostgresSqlSource) -> PostgresSqlFacts {
    let locations = locations::Locations::new(&source.sql);
    let prepared = super::parse::prepare_postgres_tokens(&source.sql);
    parsing::collect_program(source, prepared, &locations, 0, false)
}

/// Batch independent sources while preserving input order and per-source ownership.
pub fn parse_postgres_sources(sources: &[PostgresSqlSource]) -> Vec<PostgresSqlFacts> {
    use rayon::prelude::*;
    sources.par_iter().map(parse_postgres_source).collect()
}

#[cfg(test)]
mod tests;
