//! Typed INSERT, SELECT, and trigger facts from PostgreSQL SQL.
pub(super) use bounds::TableTokenIndex;

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
pub(in crate::codebase::postgres) mod value;
mod wrappers;
mod writes;

pub use crate::codebase::postgres::statement_facts::*;
pub(crate) use facts::{
    extract_from_parsed_with_recovered_placeholders, extract_sql_statement_facts_with_bounds,
    extract_sql_statement_facts_with_recovered_placeholders, project_bounds,
};
pub use facts::{extract_sql_statement_facts, has_top_level_not_exists_in};
pub use fallback::{insert_keyword_count, mask_quoted_sql};
pub(crate) use value::form_is_stable;
pub(crate) use wrappers::walk_executed;

/// Extract statement facts from SQL recovered by [`crate::codebase::postgres::extract_embedded_sql_from_source`].
///
/// This forwards the call's exact generated-placeholder positions to bound, value-form, and
/// relation-restriction facts, distinguishing interpolated binds from user-written identifiers
/// with the same marker spelling. Returns `None` when the call has no recovered SQL. Fact locations remain
/// relative to the recovered SQL text; for a `Dynamic` call, that text may contain only a
/// verified leading statement fragment.
pub fn extract_sql_statement_facts_for_embedded_call(
    call: &super::EmbeddedSqlCall,
) -> Option<SqlStatementFileFacts> {
    let sql = call.sql_text.as_deref()?;
    Some(extract_sql_statement_facts_with_recovered_placeholders(
        sql,
        true,
        &call.recovered_placeholder_positions,
    ))
}

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
