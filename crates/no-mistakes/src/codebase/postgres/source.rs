//! Standalone source facts: one prepared token inventory, one AST per statement.

mod alter;
mod columns;
mod conditional;
mod ddl;
mod drop_facts;
mod expressions;
mod generated;
mod indexes;
mod locations;
mod parsing;
mod procedural;
mod type_facts;
mod types;
pub use types::*;

/// Parse source text without a repository scan, catalog, or database connection.
pub fn parse_postgres_source(source: &PostgresSqlSource) -> PostgresSqlFacts {
    let locations = locations::Locations::new(&source.sql);
    let prepared = super::parse::prepare_postgres_tokens(&source.sql);
    parsing::collect(source, prepared, &locations)
}

/// Batch independent sources while preserving input order and per-source ownership.
pub fn parse_postgres_sources(sources: &[PostgresSqlSource]) -> Vec<PostgresSqlFacts> {
    use rayon::prelude::*;
    sources.par_iter().map(parse_postgres_source).collect()
}

#[cfg(test)]
mod tests;
