//! Typed INSERT, SELECT, and trigger facts from PostgreSQL SQL.

mod bounds;
mod conflict;
mod dedupe;
mod exists;
mod exists_correlation;
mod facts;
mod fallback;
mod insert;
mod insert_source;
mod limit;
mod lines;
mod mutations;
mod not_exists;
mod predicates;
mod select;
mod sweeps;
mod tokens;
mod trigger;
mod value;
mod wrappers;
mod writes;

pub use crate::codebase::postgres::statement_facts::*;
pub(crate) use facts::{
    extract_from_parsed_with_recovered_placeholders,
    extract_sql_statement_facts_with_recovered_placeholders,
};
pub use facts::{extract_sql_statement_facts, has_top_level_not_exists_in};
pub use fallback::{insert_keyword_count, mask_quoted_sql};
pub(crate) use value::form_is_stable;
pub(crate) use wrappers::walk_executed;

#[cfg(test)]
mod ast_coverage_tests;
#[cfg(test)]
mod builtin_form_tests;
#[cfg(test)]
mod coverage_mask_tests;
#[cfg(test)]
mod coverage_more_tests;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod value_ast_tests;
