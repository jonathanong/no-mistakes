//! Prepared PostgreSQL evaluation adapters; parsing/loading are outside timed loops.
use crate::codebase::postgres::{SchemaCatalog, SqlStatementFileFacts};

pub fn evaluate_postgres_bounds(facts: &SqlStatementFileFacts, catalog: &SchemaCatalog) -> usize {
    crate::codebase::rules::postgres_bounded_statements::benchmark_offenders(facts, catalog)
}

#[cfg(test)]
mod tests;
